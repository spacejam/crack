use libc::{c_int, syscall, SYS_gettid, pid_t,
           /* sched_yield, */ sched_setscheduler, sched_param, SCHED_FIFO};
use std::os::unix::thread::JoinHandleExt;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

fn gettid() -> pid_t {
    unsafe { syscall(SYS_gettid) as pid_t }
}

fn setscheduler(policy: c_int, priority: c_int) {
    let param = sched_param { sched_priority: priority };
    let tid = gettid();
    unsafe {
        sched_setscheduler(tid, policy, &param as *const sched_param);
    }
}

fn prioritize() {
    static PRIO: AtomicUsize = ATOMIC_USIZE_INIT;
    let prio = PRIO.fetch_add(1, Ordering::Relaxed) as c_int;
    setscheduler(SCHED_FIFO, prio);
}

#[macro_export]
macro_rules! deterministic {
    ($($thread:expr),*) => {
        let mut threads = vec![];
        $(
            prioritize();
            let thread = thread::spawn(|| {
                prioritize();
                $thread
            });
            threads.push(thread);
        )*
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }
    };
    ($($thread:expr,)*) => {
        deterministic!($($thread),*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn determined() {
        let l = Arc::new(Mutex::new(()));
        let l1 = l.clone();

        deterministic! {
            t1(l1),
            t2(l),
            t3(),
        };
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
