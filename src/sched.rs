use std::collections::{BTreeMap, HashMap, HashSet};
use std::panic;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, ThreadId, current, spawn};
use std::time::{Duration, Instant};

use ayn_rand_is_garbage::{Rng, XorShiftRng};

lazy_static! {
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}

#[derive(Debug, Default)]
struct Process {
    sleeping: BTreeMap<Instant, ThreadId>,
    clock: u64,
    runnable: HashSet<ThreadId>,
    // rng: XorShiftRng,
}

impl Process {
    fn new() -> Process {
        Process::default()
    }
}

pub struct Scheduler {
    tid_to_group: Mutex<HashMap<ThreadId, Arc<Process>>>,
}

impl Scheduler {
    pub fn run<F, T>(&self, f: F) -> T
        where F: FnOnce() -> T,
              F: panic::UnwindSafe,
              F: Send + 'static,
              T: Send + 'static
    {
        // create a new simulated process
        {
            let tid = tid();
            let process = Process::new();
            let mut ttg = self.tid_to_group.lock().unwrap();
            ttg.insert(tid, Arc::new(process));
        }

        f()
    }

    fn new() -> Scheduler {
        Scheduler {
            tid_to_group: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn sleep(&self, dur: Duration) {}

    fn register(&self) {}

    pub(crate) fn step(&self) {}

    fn panicked(&self) {}

    fn done(&self) {}

    pub fn spawn<F, T>(&self, f: F) -> JoinHandle<T>
        where F: FnOnce() -> T,
              F: panic::UnwindSafe,
              F: Send + 'static,
              T: Send + 'static
    {
        // warn if we're not under supervision

        // if we're under supervision:
        //   * register
        //   * create watcher
        //   * park thread in scheduler
        spawn(|| {
            SCHEDULER.register();
            let res = match panic::catch_unwind(f) {
                Ok(r) => r,
                Err(e) => {
                    SCHEDULER.panicked();
                    panic::resume_unwind(e)
                }
            };
            SCHEDULER.done();
            res
        })
    }
}

fn tid() -> ThreadId {
    current().id()
}
