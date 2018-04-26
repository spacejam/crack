#define  GNU_SOURCE
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/ptrace.h>
#include <sys/syscall.h>
#include <dirent.h>
#include <pthread.h>
#include <signal.h>
#include <string.h>
#include <errno.h>
#include <stdio.h>

#ifndef   THREADS
#define   THREADS  3
#endif

static int tgkill(int tgid, int tid, int sig)
{
    int retval;

    retval = syscall(SYS_tgkill, tgid, tid, sig);
    if (retval < 0) {
        errno = -retval;
        return -1;
    }

    return 0;
}

volatile unsigned long counter[THREADS + 1] = { 0UL };

volatile sig_atomic_t run = 0;
volatile sig_atomic_t done = 0;

void handle_done(int signum)
{
    done = signum;
}

int install_done(int signum)
{
    struct sigaction act;
    sigemptyset(&act.sa_mask);
    act.sa_handler = handle_done;
    act.sa_flags = 0;
    if (sigaction(signum, &act, NULL))
        return errno;
    return 0;
}

void *worker(void *data)
{
    volatile unsigned long *const counter = data;

    while (!run)
        ;

    while (!done)
        (*counter)++;

    return (void *)(*counter);
}

pid_t *gettids(const pid_t pid, size_t *const countptr)
{
    char           dirbuf[128];
    DIR           *dir;
    struct dirent *ent;

    pid_t         *data = NULL, *temp;
    size_t         size = 0;
    size_t         used = 0;

    int            tid;
    char           dummy;

    if ((int)pid < 2) {
        errno = EINVAL;
        return NULL;
    }

    if (snprintf(dirbuf, sizeof dirbuf, "/proc/%d/task/", (int)pid) >= (int)sizeof dirbuf) {
        errno = ENAMETOOLONG;
        return NULL;
    }

    dir = opendir(dirbuf);
    if (!dir)
        return NULL;

    while (1) {
        errno = 0;
        ent = readdir(dir);
        if (!ent)
            break;

        if (sscanf(ent->d_name, "%d%c", &tid, &dummy) != 1)
            continue;

        if (tid < 2)
            continue;

        if (used >= size) {
            size = (used | 127) + 129;
            temp = realloc(data, size * sizeof data[0]);
            if (!temp) {
                free(data);
                closedir(dir);
                errno = ENOMEM;
                return NULL;
            }
            data = temp;
        }

        data[used++] = (pid_t)tid;
    }
    if (errno) {
        free(data);
        closedir(dir);
        errno = EIO;
        return NULL;
    }
    if (closedir(dir)) {
        free(data);
        errno = EIO;
        return NULL;
    }

    if (used < 1) {
        free(data);
        errno = ENOENT;
        return NULL;
    }

    size = used + 1;
    temp = realloc(data, size * sizeof data[0]);
    if (!temp) {
        free(data);
        errno = ENOMEM;
        return NULL;
    }
    data = temp;

    data[used] = (pid_t)0;

    if (countptr)
        *countptr = used;

    errno = 0;
    return data;
}

int child_main(void)
{
    pthread_t   id[THREADS];
    int         i;

    if (install_done(SIGUSR1)) {
        fprintf(stderr, "Cannot set SIGUSR1 signal handler.\n");
        return 1;
    }

    for (i = 0; i < THREADS; i++)
        if (pthread_create(&id[i], NULL, worker, (void *)&counter[i])) {
            fprintf(stderr, "Cannot create thread %d of %d: %s.\n", i + 1, THREADS, strerror(errno));
            return 1;
        }

    run = 1;

    kill(getppid(), SIGUSR1);

    while (!done)
        counter[THREADS]++;

    for (i = 0; i < THREADS; i++)
        pthread_join(id[i], NULL);

    printf("Final counters:\n");
    for (i = 0; i < THREADS; i++)
        printf("\tThread %d: %lu\n", i + 1, counter[i]);
    printf("\tMain thread: %lu\n", counter[THREADS]);

    return 0;
}

int main(void)
{
    pid_t   *tid = NULL;
    size_t   tids = 0;
    int      i, k;
    pid_t    child, p;

    if (install_done(SIGUSR1)) {
        fprintf(stderr, "Cannot set SIGUSR1 signal handler.\n");
        return 1;
    }

    child = fork();
    if (!child)
        return child_main();

    if (child == (pid_t)-1) {
        fprintf(stderr, "Cannot fork.\n");
        return 1;
    }

    while (!done)
        usleep(1000);

    tid = gettids(child, &tids);
    if (!tid) {
        fprintf(stderr, "gettids(): %s.\n", strerror(errno));
        kill(child, SIGUSR1);
        return 1;
    }

    fprintf(stderr, "Child process %d has %d tasks.\n", (int)child, (int)tids);
    fflush(stderr);

    for (k = 0; k < (int)tids; k++) {
        const pid_t t = tid[k];

        if (ptrace(PTRACE_ATTACH, t, (void *)0L, (void *)0L)) {
            fprintf(stderr, "Cannot attach to TID %d: %s.\n", (int)t, strerror(errno));
            kill(child, SIGUSR1);
            return 1;
        }

        fprintf(stderr, "Attached to TID %d.\n\n", (int)t);

        fprintf(stderr, "Peeking the counters in the child process:\n");
        for (i = 0; i <= THREADS; i++) {
            long v;
            do {
                errno = 0;
                v = ptrace(PTRACE_PEEKDATA, t, &counter[i], NULL);
            } while (v == -1L && (errno == EIO || errno == EFAULT || errno == ESRCH));
            fprintf(stderr, "\tcounter[%d] = %lu\n", i, (unsigned long)v);
        }
        fprintf(stderr, "Waiting a short moment ... ");
        fflush(stderr);

        usleep(250000);

        fprintf(stderr, "and another peek:\n");
        for (i = 0; i <= THREADS; i++) {
            long v;
            do {
                errno = 0;
                v = ptrace(PTRACE_PEEKDATA, t, &counter[i], NULL);
            } while (v == -1L && (errno == EIO || errno == EFAULT || errno == ESRCH));
            fprintf(stderr, "\tcounter[%d] = %lu\n", i, (unsigned long)v);
        }
        fprintf(stderr, "\n");
        fflush(stderr);

        usleep(250000);

        ptrace(PTRACE_DETACH, t, (void *)0L, (void *)0L);
    }

    for (k = 0; k < 4; k++) {
        const pid_t t = tid[tids / 2];

        if (k == 0) {
            fprintf(stderr, "Sending SIGSTOP to child process ... ");
            fflush(stderr);
            kill(child, SIGSTOP);
        } else
        if (k == 1) {
            fprintf(stderr, "Sending SIGCONT to child process ... ");
            fflush(stderr);
            kill(child, SIGCONT);
        } else
        if (k == 2) {
            fprintf(stderr, "Sending SIGSTOP to TID %d ... ", (int)tid[0]);
            fflush(stderr);
            tgkill(child, tid[0], SIGSTOP);
        } else
        if (k == 3) {
            fprintf(stderr, "Sending SIGCONT to TID %d ... ", (int)tid[0]);
            fflush(stderr);
            tgkill(child, tid[0], SIGCONT);
        }
        usleep(250000);
        fprintf(stderr, "done.\n");
        fflush(stderr);

        if (ptrace(PTRACE_ATTACH, t, (void *)0L, (void *)0L)) {
            fprintf(stderr, "Cannot attach to TID %d: %s.\n", (int)t, strerror(errno));
            kill(child, SIGUSR1);
            return 1;
        }

        fprintf(stderr, "Attached to TID %d.\n\n", (int)t);

        fprintf(stderr, "Peeking the counters in the child process:\n");
        for (i = 0; i <= THREADS; i++) {
            long v;
            do {
                errno = 0;
                v = ptrace(PTRACE_PEEKDATA, t, &counter[i], NULL);
            } while (v == -1L && (errno == EIO || errno == EFAULT || errno == ESRCH));
            fprintf(stderr, "\tcounter[%d] = %lu\n", i, (unsigned long)v);
        }
        fprintf(stderr, "Waiting a short moment ... ");
        fflush(stderr);

        usleep(250000);

        fprintf(stderr, "and another peek:\n");
        for (i = 0; i <= THREADS; i++) {
            long v;
            do {
                errno = 0;
                v = ptrace(PTRACE_PEEKDATA, t, &counter[i], NULL);
            } while (v == -1L && (errno == EIO || errno == EFAULT || errno == ESRCH));
            fprintf(stderr, "\tcounter[%d] = %lu\n", i, (unsigned long)v);
        }
        fprintf(stderr, "\n");
        fflush(stderr);

        usleep(250000);

        ptrace(PTRACE_DETACH, t, (void *)0L, (void *)0L);
    }

    kill(child, SIGUSR1);

    do {
        p = waitpid(child, NULL, 0);
        if (p == -1 && errno != EINTR)
            break;
    } while (p != child);

    return 0;
}
