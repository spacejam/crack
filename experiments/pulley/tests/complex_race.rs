#[macro_use]
extern crate crack;

use crack::{Mutex, AtomicUsize};
use std::cell::UnsafeCell;
use std::sync::Mutex;

fn t1(ss: &SharedState) {
    for _ in 0..2 {
        step!();
        let _ = ss.mu.lock().unwrap();
        unsafe {
            step!();
            *ss.x.get() = 1;
            step!();
            *ss.y.get() = 1;
        }
        step!();
        // implicit unlock
    }
}

fn t2(ss: &SharedState) {
    for _ in 0..2 {
        {
            step!();
            let _ = ss.mu.lock().unwrap();
            unsafe {
                step!();
                *ss.x.get() = 0;
            }
            step!();
            // implicit unlock
        }
        unsafe {
            step!();
            if *ss.x.get() > 0 {
                step!();
                *ss.y.get() += 1;
                step!();
                *ss.x.get() = 2;
            }
        }
    }
}

fn t3(ss: &SharedState) {
    for _ in 0..2 {
        unsafe {
            step!();
            if *ss.x.get() > 1 {
                step!();
                if *ss.y.get() == 3 {
                    panic!("ahhh wtf");
                } else {
                    step!();
                    *ss.y.get() = 2;
                }
            }
        }
    }
}

#[derive(Debug)]
struct SharedState {
    mu: Mutex<()>,
    x: UnsafeCell<usize>,
    y: UnsafeCell<usize>,
}

#[test]
fn complex_race() {
    let l = || {
        SharedState {
            mu: Mutex::new(()),
            x: UnsafeCell::new(0),
            y: UnsafeCell::new(0),
        }
    };

    let mut scheduler = crack::Scheduler::with_initializer(l);

    scheduler.add(t1);
    scheduler.add(t2);
    scheduler.add(t3);

    // scheduler.run_with_seed(20852);
    scheduler.explore();
}
