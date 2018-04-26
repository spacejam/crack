#!/usr/bin/gdb -x

'''
a simple python GDB script for running multithreaded
programs in a way that is 'deterministic enough'
to tease out and replay interesting bugs.

Tyler Neely 25 Sept 2017
t@jujit.su

references:
    https://sourceware.org/gdb/onlinedocs/gdb/All_002dStop-Mode.html
    https://sourceware.org/gdb/onlinedocs/gdb/Non_002dStop-Mode.html
    https://sourceware.org/gdb/onlinedocs/gdb/Threads-In-Python.html
    https://sourceware.org/gdb/onlinedocs/gdb/Events-In-Python.html
    https://blog.0x972.info/index.php?tag=gdb.py
'''

import gdb
import random
import json
import sys

###############################################################################
#                                   config                                    #
###############################################################################
# options are 'random' and 'exhaustive'
mode = 'exhaustive'

# exits 1 when an invariant is violated, rather than leaving gdb open
exit_on_violation = False

# set this to a number for reproducing results in random mode
seed = None # None  # 951931004895

# If in exhaustive mode, and our threads are all basically the same for
# the purposes of this test, we can perform fewer executions for the same
# effect.
symmetric_threads = True

# set this to the number of valid threads in the program
# {2, 3} assumes a main thread that waits on 2 workers.
# {1, ... N} assumes all of the first N threads are to be explored
threads_whitelist = {2, 3}

# set this to the file of the binary to explore
filename = 'target/debug/race'

# set this to the place the threads should rendezvous before exploring
entrypoint = 'src/main.rs:9'

# set this to after the threads are done
exitpoint = 'src/main.rs:14'

# invariant unreachable points that should never be accessed
unreachable = [
        'panic_unwind::imp::panic'
        ]

# set this to the locations you want to test interleavings for
interesting = [
        'src/main.rs:9',
        'src/main.rs:10'
        ]

# this is the file that is incrementally built to record explored paths
state_file = 'execution_explorer.state.json'

# uncomment this to output the specific commands issued to gdb
gdb.execute('set trace-commands on')

###############################################################################
#                              execution logic                                #
###############################################################################


class UnreachableBreakpoint(gdb.Breakpoint):
    """Mark unreachable code that will trigger an exit."""
    pass


class DoneBreakpoint(gdb.Breakpoint):
    """Mark the exit point that threads drain into when done."""
    pass


class InterestingBreakpoint(gdb.Breakpoint):
    """Mark an interesting place for exploring different interleavings at."""
    pass


class DeterministicExecutor:
    """Run threaded code deterministically in various ways."""
    def __init__(self, seed=None, mode='exhaustive', symmetric_threads=True):
        self.seed = seed
        self.mode = mode                             # 'random' or 'exhaustive'
        self.symmetric_threads = symmetric_threads   # threads are identical

        # internal state
        self.state = {}                              # thread choice history
        self.ready = set()                           # ready threads
        self.finished = set()                        # finished threads
        self.path = []                               # the thread schedule path

        self.load()

        # pick a random seed if one isn't provided
        if self.seed:
            random.seed(self.seed)
        else:
            self.reseed()
        print('seeding with', seed)

        gdb.execute('file ' + filename)

        # non-stop is necessary to provide thread-specific
        # information when breakpoints are hit.
        gdb.execute('set non-stop on')

        # (not necessary) don't ask for confirmation if we end up
        # in a shell through some bug.
        gdb.execute('set confirm off')

    def load(self):
        # try to load previous state
        try:
            with open(state_file, 'r') as sf:
                self.state = json.loads(sf.read())
                if 'seed' in self.state:
                    self.seed = self.state['seed']
                print("loaded existing state, continuing where it left off")
        except Exception as e:
            print("unable to read or deserialize existing state file:", e)
            pass

    def save(self):
        try:
            with open(state_file, 'tw') as sf:
                sf.write(json.dumps(self.state))
        except Exception as e:
            print("unable to open, write or serialize state to file:", e)
            pass

    def reseed(self):
        random.seed()
        self.seed = random.randrange(1e12)
        print('reseeding with', self.seed)
        random.seed(self.seed)

    def rendezvous_callback(self, event):
        try:
            self.ready.add(event.inferior_thread.num)
            if len(self.ready) == len(threads_whitelist):
                # all expected threads are ready, kick off our workload!
                self.run_schedule()
        except Exception as e:
            # this will be thrown if breakpoint is not a part of event,
            # like when the event was stopped for another reason.
            print(e)

    def run(self):
        gdb.execute('b ' + entrypoint)

        gdb.events.stop.connect(self.rendezvous_callback)
        gdb.events.exited.connect(self.exit_callback)

        gdb.execute('r')

    def run_schedule(self):
        print('running schedule')
        gdb.execute('d')
        gdb.events.stop.disconnect(self.rendezvous_callback)
        gdb.events.stop.connect(self.scheduler_callback)

        for bp in interesting:
            InterestingBreakpoint(bp)

        for bp in unreachable:
            UnreachableBreakpoint(bp)

        DoneBreakpoint(exitpoint)

        self.pick()

    def scheduler_callback(self, event):
        if not isinstance(event, gdb.BreakpointEvent):
            print('WTF sched callback got', event.__dict__)
            return

        if isinstance(event.breakpoint, DoneBreakpoint):
            print("thread", gdb.selected_thread().num, "done at", event.breakpoint.location)
            self.finished.add(event.inferior_thread.num)
        elif isinstance(event.breakpoint, UnreachableBreakpoint):
            print('!' * 80)
            print('unreachable breakpoint triggered with seed', self.seed)
            print('!' * 80)
            gdb.events.exited.disconnect(self.exit_callback)
            if exit_on_violation:
                sys.exit(1)
        else:
            print('thread', event.inferior_thread.num,
                  'hit breakpoint at', event.breakpoint.location)

        self.pick()

    def pick(self):
        """
        The main scheduling logic gets called here. This
        is called at the beginning of the run, and when
        a watched thread hits an InterestingBreakpoint.
        """
        threads = self.runnable_threads()
        if not threads:
            print('threads all done')
            self.save()
            gdb.execute('q')

        if self.mode == 'random':
            thread = random.choice(threads)
        elif self.mode == 'exhaustive':
            thread = self.bfs(threads)
        else:
            print('self.mode must be either \'exhaustive\' or \'random\'')
            gdb.execute('q')

        gdb.execute('t ' + thread)
        gdb.execute('c')

    def bfs(self, threads):
        """
        We traverse the decision tree of scheduled threads in
        a BFS way to improve branch diversity over time.
        """
        if 'paths' not in self.state:
            self.state['paths'] = {}
        explored = self.state['paths']

        if self.symmetric_threads and len(self.path) == 0:
            # if symmetric, we can get away with
            # a single root choice to cut execution
            # to 1/N of a full enumeration.
            if len(explored) is 0:
                # pick initial thread
                print("choosing original")
            elif len(explored) is 1:
                # pick the last thread
                print("choosing saved original from", explored.keys())
                for thread in explored.keys():
                    self.path.append(thread)
                    return thread
            else:
                print("running in symmetric mode but we've \
                        somehow chosen multiple root threads!")
                gdb.execute('q')

        # shift cursor up a bit
        for choice in self.path:
            explored = explored[choice]

        # pick a thread that we have not yet chosen before
        unexplored = [choice for choice in threads
                      if choice not in explored]

        if not unexplored:
            print("all interesting permutations explored.")
            sys.exit(0)
        thread = random.choice(unexplored)
        self.path.append(thread)
        explored[thread] = {}
        return thread

    def runnable_threads(self):
        threads = gdb.selected_inferior().threads()

        def f(it):
            return (it.is_valid() and not
                    it.is_exited() and
                    it.num in threads_whitelist and
                    it.num not in self.finished)

        good_threads = [str(it.num) for it in threads if f(it)]
        good_threads.sort()

        return good_threads

    def exit_callback(self, event):
        try:
            if event.exit_code != 0:
                print('!' * 80)
                if self.mode == 'random':
                    print('interesting exit with seed', self.seed)
                elif self.mode == 'exhaustive':
                    print('interesting exit with saved path')
                print('!' * 80)

                if exit_on_violation:
                    sys.exit(1)
            else:
                print('happy exit')
                self.save()

            gdb.execute('q')
        except Exception as e:
            pass

de = DeterministicExecutor(seed, mode, symmetric_threads)
de.run()
