use std::ops::{Deref, DerefMut};
use std::sync::{LockResult as StdLockResult, Mutex as StdMutex,
                MutexGuard as StdMutexGuard, PoisonError, RwLock as StdRwLock,
                RwLockReadGuard as StdRwLockReadGuard,
                RwLockWriteGuard as StdRwLockWriteGuard, TryLockError,
                TryLockResult as StdTryLockResult};
use std::sync::atomic::AtomicUsize as RAtomicUsize;

use sched::SCHEDULER;

pub struct Mutex<T> {
    inner: StdMutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Mutex<T> {
        Mutex {
            inner: StdMutex::new(t),
        }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<T>, StdMutexGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.lock()?;

        Ok(MutexGuard {
            inner: guard,
        })
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<T>, StdMutexGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.try_lock()?;

        Ok(MutexGuard {
            inner: guard,
        })
    }

    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }
}

pub struct MutexGuard<'a, T: 'a> {
    inner: StdMutexGuard<'a, T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

pub struct RwLock<T> {
    inner: StdRwLock<T>,
}

impl<T> RwLock<T> {
    pub fn new(t: T) -> RwLock<T> {
        RwLock {
            inner: StdRwLock::new(t),
        }
    }

    pub fn read(
        &self,
    ) -> LockResult<RwLockReadGuard<T>, StdRwLockReadGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.read()?;

        // NB we step twice in RwLock's
        SCHEDULER.step();

        Ok(RwLockReadGuard {
            inner: guard,
        })
    }

    pub fn try_read(
        &self,
    ) -> TryLockResult<RwLockReadGuard<T>, StdRwLockReadGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.try_read()?;

        // NB we step twice in RwLock's
        SCHEDULER.step();

        Ok(RwLockReadGuard {
            inner: guard,
        })
    }

    pub fn write(
        &self,
    ) -> LockResult<RwLockWriteGuard<T>, StdRwLockWriteGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.write()?;

        Ok(RwLockWriteGuard {
            inner: guard,
        })
    }

    pub fn try_write(
        &self,
    ) -> TryLockResult<RwLockWriteGuard<T>, StdRwLockWriteGuard<T>> {
        SCHEDULER.step();

        let guard = self.inner.try_write()?;

        Ok(RwLockWriteGuard {
            inner: guard,
        })
    }
}

pub struct RwLockReadGuard<'a, T: 'a> {
    inner: StdRwLockReadGuard<'a, T>,
}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

pub struct RwLockWriteGuard<'a, T: 'a> {
    inner: StdRwLockWriteGuard<'a, T>,
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

pub type LockResult<A, B> = Result<A, PoisonError<B>>;

pub type TryLockResult<A, B> = Result<A, TryLockError<B>>;
