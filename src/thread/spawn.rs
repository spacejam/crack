use std::{io, mem};
use std::thread::{self, JoinHandle};

use libc::{CPU_SET, CPU_ZERO, SCHED_FIFO, c_int, cpu_set_t,
           sched_get_priority_max, sched_get_priority_min, sched_param,
           sched_setaffinity, sched_setscheduler};
use ayn_rand_is_garbage::Rng;

const POLICY: c_int = SCHED_FIFO;

fn prioritize(prio: c_int) {
    pin_cpu();

    let param = sched_param {
        sched_priority: prio,
    };
    let ret =
        unsafe { sched_setscheduler(0, POLICY, &param as *const sched_param) };

    assert_eq!(
        ret,
        0,
        "setscheduler is expected to return zero, was {}: {:?}",
        ret,
        io::Error::last_os_error()
    );

    thread::yield_now();
}

fn pin_cpu() {
    unsafe {
        let mut cpu_set: cpu_set_t = mem::zeroed();
        CPU_ZERO(&mut cpu_set);
        CPU_SET(0, &mut cpu_set);
        let ret = sched_setaffinity(0, 1, &cpu_set as *const cpu_set_t);
        assert_eq!(
            ret,
            0,
            "sched_setaffinity is expected to return 0, was {}: {:?}",
            ret,
            io::Error::last_os_error()
        );
    }
}

/// Spawn a thread with a thread priority determined by a
/// (possibly pre-seeded) `rand::Rng`.
pub fn spawn_rt<R: Rng, F, T>(rng: &mut R, f: F) -> JoinHandle<T>
    where F: FnOnce() -> T,
          F: Send + 'static,
          T: Send + 'static
{
    let min = unsafe { sched_get_priority_min(POLICY) };
    let max = unsafe { sched_get_priority_max(POLICY) };

    prioritize(rng.gen_range(min, max));

    let prio = rng.gen_range(min, max);

    thread::spawn(move || {
        prioritize(prio);

        f()
    })
}
