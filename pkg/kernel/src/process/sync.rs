use alloc::{collections::BTreeMap, vec::Vec};
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

/// Mutex is provided by the sem manager,
/// We need not to protect the count again
pub struct Semaphore {
    count: usize,
    wait_queue: Vec<ProcessId>,
}

impl Semaphore {
    pub fn new(value: usize) -> Self {
        Self {
            count: value,
            wait_queue: Vec::new()
        }
    }

    pub fn down(&mut self, pid: ProcessId) -> Result<(),()> {
        if self.count == 0 {
            self.wait_queue.push(pid);
            Err(())
        } else {
            self.count -= 1;
            // trace!("Semaphore down: {}", count);
            Ok(())
        }
    }

    pub fn up(&mut self) -> Option<ProcessId> {
        // trace!("Semaphore up: {}", count);
        if !self.wait_queue.is_empty() {
            Some(self.wait_queue.pop().unwrap())
        } else {
            self.count += 1;
            None
        }
    }
}

impl core::fmt::Display for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Semaphore({}) {:?}", self.count, self.wait_queue)
    }
}
