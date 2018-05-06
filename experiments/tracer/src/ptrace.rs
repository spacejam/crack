use libc::{PTRACE_ATTACH, PTRACE_CONT, PTRACE_O_TRACECLONE,
           PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_SINGLESTEP,
           PTRACE_TRACEME, SIGSTOP, SYS_gettid, WUNTRACED, c_int, c_uint,
           fork, pid_t, ptrace, raise, syscall, waitpid};
use std::io::Error;
use std::path::Path;

fn tracee<F>(f: F)
    where F: Fn() -> ()
{
    unsafe {
        ptrace(PTRACE_TRACEME, 0, 0, 0);
        println!("child {} stopped", syscall(SYS_gettid));
        raise(SIGSTOP);
        println!("child resumed");
        f();
        println!("child exiting");
    }
}

fn tracer(child: pid_t) {
    unsafe {
        let res = waitpid(child, 0 as *mut i32, WUNTRACED);
        println!("waitpid: {}", res);
    }

    let task_dir = format!("/proc/{}/task", child);
    let mut tasks = vec![];
    let entries = Path::new(&task_dir).read_dir().expect(
        "could not read child's proc directory",
    );

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let path_str = file_name.to_string_lossy();
            if let Ok(id) = path_str.parse::<c_uint>() {
                tasks.push(id);
            }
        }
    }

    println!("got tasks {:?} from entries in {}", tasks, task_dir);
    for tid in tasks {
        unsafe {
            let options = PTRACE_O_TRACESYSGOOD | PTRACE_O_TRACECLONE;
            let res = ptrace(PTRACE_SETOPTIONS, tid, 0, options);
            assert_eq!(res, 0, "messed up errno: {:?}", Error::last_os_error());
            for _ in 0..100 {
                print!("s");
                ptrace(PTRACE_CONT, tid, 0, 0);
                waitpid(child, 0 as *mut i32, WUNTRACED);
                // std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }


}

fn serialize<F>(f: F)
    where F: Fn() -> ()
{
    let child = unsafe { fork() };
    if child == 0 { tracee(f) } else { tracer(child) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    #[ignore]
    fn ptraced() {
        serialize(|| {
            let l = Arc::new(Mutex::new(()));
            let l1 = l.clone();

            let t1 = thread::spawn(|| t1(l1));
            let t2 = thread::spawn(|| t2(l));
            let t3 = thread::spawn(t3);

            for t in vec![t1, t2, t3].into_iter() {
                t.join();
            }
        })
    }

    static mut X: u8 = 0;
    static mut Y: u8 = 0;

    fn t1(l: Arc<Mutex<()>>) {
        for _ in 0..2 {
            println!("t1");
            let _ = l.lock().unwrap();
            unsafe {
                X = 1;
                Y = 1;
            }
        }
    }

    fn t2(l: Arc<Mutex<()>>) {
        for _ in 0..2 {
            println!("t2");
            {
                let _ = l.lock().unwrap();
                unsafe {
                    X = 0;
                }
            }
            // mutex unlocked
            unsafe {
                if X > 0 {
                    Y += 1;
                    X = 2;
                }
            }
        }
    }

    fn t3() {
        unsafe {
            if X > 1 {
                if Y == 3 {
                    panic!("bug reached!");
                } else {
                    Y = 2;
                }
            }
        }
        println!("t3");
    }
}
