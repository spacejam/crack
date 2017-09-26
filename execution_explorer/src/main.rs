use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

static A: AtomicUsize = ATOMIC_USIZE_INIT;

/// Try to add 10 to A iff its value is 0.
fn bad_txn() {
    if A.load(SeqCst) == 0 {
        A.fetch_add(10, SeqCst);
    }
    assert_eq!(A.load(SeqCst), 10);
    println!("yee");
}

fn main() {
    let mut threads = vec![];

    for _ in 0..2 {
        threads.push(thread::spawn(|| bad_txn()));
    }

    for handle in threads.into_iter() {
        handle.join().unwrap();
    }
}
