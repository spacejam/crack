use std::sync::{Arc, Mutex};
use std::thread;

static mut X: i32 = 0;
static mut Y: i32 = 0;

fn t1(l: Arc<Mutex<()>>) {
    for _ in 0..2 {
        l.lock().unwrap();
        unsafe {
            X = 1;
            Y = 1;
        }
    }
}

fn t2(l: Arc<Mutex<()>>) {
    for _ in 0..2 {
        {
            l.lock().unwrap();
            unsafe {
                X = 0;
            }
        }
        unsafe {
            if X > 0 {
                Y += 1;
                X = 2;
            }
        }
    }
}

fn t3() {
    for _ in 0..2 {
        unsafe {
            if X > 1 {
                if Y == 3 {
                    panic!("ahhh wtf");
                } else {
                    Y = 2;
                }
            }
        }
    }
}

fn main() {
    let l = Arc::new(Mutex::new(()));
    let l2 = l.clone();
    let threads = vec![
        thread::spawn(move || t1(l)),
        thread::spawn(move || t2(l2)),
        thread::spawn(move || t3()),
    ];
    for t in threads.into_iter() {
        t.join().unwrap();
    }
}
