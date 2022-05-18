use alloc::{collections::BTreeMap, vec::Vec};
use spin::Mutex;
use super::ProcessId;

once_mutex!(pub SEMAPHORES: BTreeMap<SemaphoreId, Semaphore>);

guard_access_fn!{
    pub get_sem_manager(SEMAPHORES: BTreeMap<SemaphoreId, Semaphore>)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SemaphoreId(u32);

impl SemaphoreId {
    pub fn new(key: u32) -> Self {
        Self(key)
    }
}

pub struct Semaphore {
    count: Mutex<u64>,
    wait_queue: Vec<ProcessId>,
}

impl Semaphore {
    pub fn new() -> Self {
        Self {
            count: Mutex::new(1),
            wait_queue: Vec::new()
        }
    }

    pub fn down(&mut self, pid: ProcessId) -> Result<(),()> {
        if let Some(mut count) = self.count.try_lock() {
            if *count == 0 {
                self.wait_queue.push(pid);
                Err(())
            } else {
                *count -= 1;
                // trace!("Semaphore down: {}", count);
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn up(&mut self) -> Option<ProcessId> {
        if let Some(mut count) = self.count.try_lock() {
            // trace!("Semaphore up: {}", count);
            if !self.wait_queue.is_empty() {
                Some(self.wait_queue.pop().unwrap())
            } else {
                *count += 1;
                None
            }
        } else {
            None
        }
    }
}

impl core::fmt::Display for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Semaphore({}) {:?}", self.count.lock(), self.wait_queue)
    }
}
