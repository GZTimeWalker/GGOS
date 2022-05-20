use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::*;

pub struct SpinLock {
    bolt: AtomicBool,
}

impl SpinLock {
    pub const INIT: Self = Self {
        bolt: AtomicBool::new(false),
    };

    pub const fn new() -> Self {
        Self::INIT
    }

    pub fn lock(&mut self) {
        while self
            .bolt
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }
    }

    pub fn unlock(&mut self) {
        self.bolt.store(false, Ordering::Relaxed);
    }
}

unsafe impl Sync for SpinLock {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore(pub u32);

impl Semaphore {
    pub fn new(key: u32) -> Self {
        Semaphore(key)
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> isize {
        sys_new_sem(self.0, value)
    }

    /// use after init
    #[inline(always)]
    pub fn release(&self) {
        while sys_sem_up(self.0) != 0 {}
    }

    /// use after init
    #[inline(always)]
    pub fn acquire(&self) {
        while sys_sem_down(self.0) != 0 {}
    }

    /// use after init
    #[inline(always)]
    pub fn free(&self) -> isize {
        sys_rm_sem(self.0)
    }
}

unsafe impl Sync for Semaphore {}

#[macro_export]
macro_rules! semaphore_array {
    [$($x:expr),+ $(,)?] => {
        [ $($crate::Semaphore($x),)* ]
    }
}
