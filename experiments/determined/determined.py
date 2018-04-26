#!/usr/bin/gdb -x

'''
a simple python GDB script for running multithreaded
programs pseudodeterministically

Tyler Neely 4 Jan 2018
t@jujit.su

references:
    https://sourceware.org/gdb/onlinedocs/gdb/All_002dStop-Mode.html
    https://sourceware.org/gdb/onlinedocs/gdb/Non_002dStop-Mode.html
    https://sourceware.org/gdb/onlinedocs/gdb/Threads-In-Python.html
    https://sourceware.org/gdb/onlinedocs/gdb/Events-In-Python.html
    https://blog.0x972.info/index.php?tag=gdb.py
'''

import random
import json
import sys

import gdb

###############################################################################
#                                   config                                    #
###############################################################################
config = {
    'seed': 0,

    # set this to the file of the binary to explore
    "filename": "thing",

    # invariant unreachable points that should never be accessed
    "unreachable": ["panic_unwind::imp::panic"],

    # set this to the locations you want to test interleavings for
    'interesting': [
        "time.Sleep",
        "sync.runtime_Semacquire",
        "sync.runtime_SemacquireMutex",
        "sync.runtime_Semrelease",
    ],

    # print out all gdb commands
    'trace': True
}

###############################################################################
#                              execution logic                                #
###############################################################################


class UnreachableBreakpoint(gdb.Breakpoint):
    """Mark unreachable code that will trigger an exit."""
    pass


class InterestingBreakpoint(gdb.Breakpoint):
    """Mark an interesting place for exploring different interleavings at."""
    pass


class DeterministicExecutor:
    """Run threaded code deterministically in various ways."""
    def __init__(self):
        self.seed = config['seed']

        # pick a random seed if one isn't provided
        if self.seed:
            print('seeding with', self.seed)
            random.seed(self.seed)

        gdb.execute('file ' + config['filename'])

        # non-stop is necessary to provide thread-specific
        # information when breakpoints are hit.
        # gdb.execute('set non-stop on')

        # (not necessary) don't ask for confirmation if we end up
        # in a shell through some bug.
        gdb.execute('set confirm off')

        # print out which commands are executed as they happen
        if config['trace']:
            gdb.execute('set trace-commands on')

    def run(self):
        gdb.execute('b main')

        gdb.events.stop.connect(self.scheduler_callback)
        gdb.events.exited.connect(self.exit_callback)

        for bp in config['interesting']:
            InterestingBreakpoint(bp)

        for bp in config['unreachable']:
            UnreachableBreakpoint(bp)

        gdb.execute('r')
        gdb.execute('set scheduler-locking on')

        self.pick()

    def scheduler_callback(self, event):
        if not isinstance(event, gdb.BreakpointEvent):
            print('WTF sched callback got', event.__dict__)
            return

        print("hit", event.breakpoint.location)
        if isinstance(event.breakpoint, UnreachableBreakpoint):
            print('!' * 80)
            print('unreachable breakpoint triggered with seed', self.seed)
            print('!' * 80)
            gdb.events.exited.disconnect(self.exit_callback)
            if config['exit on violation']:
                sys.exit(1)

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
            gdb.execute('q')

        thread = random.choice(threads)
        print('executing thread ', thread)
        gdb.execute('t ' + thread)
        gdb.execute('c')

    def runnable_threads(self):
        threads = gdb.selected_inferior().threads()

        def f(it):
            return it.is_valid() and not it.is_exited()

        good_threads = [str(it.num) for it in threads if f(it)]
        good_threads.sort()

        return good_threads

    def exit_callback(self, event):
        try:
            if event.exit_code != 0:
                print('!' * 80)
                print('interesting exit with seed', self.seed)
                print('!' * 80)

                if config['exit on violation']:
                    sys.exit(1)
            else:
                print('happy exit')

            gdb.execute('q')
        except Exception as e:
            pass


de = DeterministicExecutor()
de.run()
