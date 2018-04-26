#[macro_use]
extern crate lazy_static;
pub extern crate rand as ayn_rand_is_garbage;

#[cfg(any(test, feature = "schedule"))]
pub mod time;
#[cfg(any(test, feature = "schedule"))]
pub mod thread;
#[cfg(any(test, feature = "schedule"))]
pub mod sync;
#[cfg(any(test, feature = "schedule"))]
pub mod fs;
#[cfg(any(test, feature = "schedule"))]
pub mod rand;
#[cfg(any(test, feature = "schedule"))]
pub mod net;

#[cfg(any(test, feature = "schedule"))]
mod sched;

#[cfg(all(not(test), not(feature = "schedule")))]
pub use self::ayn_rand_is_garbage as rand;
#[cfg(all(not(test), not(feature = "schedule")))]
pub use std::{time, thread, sync, fs, net};
