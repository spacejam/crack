extern crate rand;

use rand::{Rng, SeedableRng, StdRng};
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

thread_local! {
    pub static HANDLE: RefCell<Option<Handle>> = RefCell::new(None);
}

#[macro_export]
macro_rules! step {
    ($label:expr) => {
        $crate::HANDLE.with(|h| if let Some(ref handle) = *h.borrow() {
            handle.step();
        } else {
            panic!("step called outside of a scheduled context");
        });
    };
    () => {
        step!(format!("%s:%s", module_path!(), line!()));
    }
}

pub type ScheduledFn<State> = fn(&State);

#[derive(Debug, PartialEq)]
enum SchedulerMessage {
    Rendezvous,
    Step,
    Exit,
}

fn handle_pair() -> (SchedulerHandle, Handle) {
    let (to_thread, from_sched) = sync_channel(0);
    let (to_sched, from_thread) = sync_channel(0);

    let sh = SchedulerHandle {
        to_thread: to_thread,
        from_thread: from_thread,
    };

    let h = Handle {
        to_sched: to_sched,
        from_sched: from_sched,
    };

    (sh, h)
}

struct SchedulerHandle {
    to_thread: SyncSender<()>,
    from_thread: Receiver<SchedulerMessage>,
}

pub struct Handle {
    to_sched: SyncSender<SchedulerMessage>,
    from_sched: Receiver<()>,
}

impl Handle {
    fn bind(self) {
        self.to_sched.send(SchedulerMessage::Rendezvous).unwrap();
        self.from_sched.recv().unwrap();
        HANDLE.with(|h| *h.borrow_mut() = Some(self));
    }

    pub fn step(&self) {
        self.to_sched.send(SchedulerMessage::Step).unwrap();
        self.from_sched.recv().unwrap();
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.to_sched.send(SchedulerMessage::Exit).unwrap();
    }
}

struct PtrHack<State>(*mut State);

unsafe impl<State> Send for PtrHack<State> {}
unsafe impl<State> Sync for PtrHack<State> {}

impl<State> Clone for PtrHack<State> {
    fn clone(&self) -> PtrHack<State> {
        PtrHack(self.0)
    }
}

pub struct Scheduler<State> {
    targets: Vec<ScheduledFn<State>>,
    initializer: fn() -> State,
}

impl<State> Scheduler<State>
    where State: 'static
{
    pub fn with_initializer(f: fn() -> State) -> Scheduler<State> {
        Scheduler {
            targets: vec![],
            initializer: f,
        }
    }

    pub fn add(&mut self, f: ScheduledFn<State>) {
        self.targets.push(f);
    }

    pub fn explore(self) {
        for i in 0..100_000 {
            self.run_with_seed(i);
        }
    }

    pub fn run_with_seed(&self, seed: usize) {
        println!("----- seeding with {}", seed);
        let mut rng = StdRng::from_seed(&[seed]);

        // initialize threads
        let state = PtrHack(Box::into_raw(Box::new((self.initializer)())));
        let mut sched_handles = vec![];
        let mut threads = vec![];

        let targets = self.targets.clone();
        for target in targets {
            let (sched_handle, handle) = handle_pair();
            let state = state.clone();

            let thread = std::thread::spawn(move || {
                handle.bind();
                unsafe {
                    (target)(&*state.0);
                }
            });

            assert_eq!(
                sched_handle.from_thread.recv().unwrap(),
                SchedulerMessage::Rendezvous
            );

            sched_handles.push(sched_handle);
            threads.push(thread);
        }

        // pick a random one until all are done
        while !sched_handles.is_empty() {
            let choice: usize = rng.gen_range(0, sched_handles.len());
            // println!("choice: {}", choice);
            let should_drop = {
                let sh = &sched_handles[choice];
                sh.to_thread.send(()).unwrap();
                match sh.from_thread.recv().unwrap() {
                    SchedulerMessage::Step => false,
                    SchedulerMessage::Exit => true,
                    SchedulerMessage::Rendezvous => {
                        panic!("got Rendezvous while stepping thread")
                    }
                }
            };
            if should_drop {
                sched_handles.remove(choice);
                if let Err(_) = threads.remove(choice).join() {
                    panic!("failed test using seed {}", seed);
                }
                //println!("ended thread {}", choice);
            }
        }

        unsafe {
            drop(Box::from_raw(state.0));
        }
    }
}
