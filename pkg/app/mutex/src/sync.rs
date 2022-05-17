use core::{sync::atomic::{AtomicBool, Ordering}, hint::spin_loop};

pub struct SpinLock {
    bolt: AtomicBool
}

impl SpinLock {
    pub const INIT: Self = Self {
        bolt: AtomicBool::new(false)
    };

    pub const fn new() -> Self {
        Self::INIT
    }

    pub fn lock(&mut self) {
        while self.bolt.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            spin_loop();
        }
    }

    pub fn unlock(&mut self) {
        self.bolt.store(false, Ordering::Relaxed);
    }
}
