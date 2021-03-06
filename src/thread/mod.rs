use std::panic;
use std::thread::{JoinHandle, ThreadId, current};
use std::time::Duration;

use sched::SCHEDULER;

#[cfg(target_os = "linux")]
mod spawn;
#[cfg(target_os = "linux")]
pub use self::spawn::spawn_rt;

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    where F: FnOnce() -> T,
          F: panic::UnwindSafe,
          F: Send + 'static,
          T: Send + 'static
{
    SCHEDULER.spawn(f)
}

pub fn sleep(dur: Duration) {
    SCHEDULER.sleep(dur);
}

pub fn sleep_ms(ms: u32) {
    sleep(Duration::from_millis(ms as u64))
}
