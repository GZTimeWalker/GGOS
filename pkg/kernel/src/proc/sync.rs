use super::ProcessId;
use alloc::{collections::BTreeMap, vec::Vec};
use spin::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SemaphoreId(u32);

impl SemaphoreId {
    pub fn new(key: u32) -> Self {
        Self(key)
    }
}

/// Mutex is required for Semaphore
#[derive(Debug, Clone)]
pub struct Semaphore {
    count: usize,
    wait_queue: Vec<ProcessId>,
}

/// Semaphore result
#[derive(Debug)]
pub enum SemaphoreResult {
    Ok,
    NoExist,
    Block(ProcessId),
    WakeUp(ProcessId),
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(value: usize) -> Self {
        Self {
            count: value,
            wait_queue: Vec::new(),
        }
    }

    /// Down the semaphore (acquire)
    ///
    /// if the count is 0, then push the process into the wait queue
    /// else decrease the count and return Ok
    pub fn down(&mut self, pid: ProcessId) -> SemaphoreResult {
        if self.count == 0 {
            self.wait_queue.push(pid);
            SemaphoreResult::Block(pid)
        } else {
            self.count -= 1;
            SemaphoreResult::Ok
        }
    }

    /// Up the semaphore (release)
    ///
    /// if the wait queue is not empty, then pop a process from the wait queue
    /// else increase the count
    pub fn up(&mut self) -> SemaphoreResult {
        if !self.wait_queue.is_empty() {
            SemaphoreResult::WakeUp(self.wait_queue.pop().unwrap())
        } else {
            self.count += 1;
            SemaphoreResult::Ok
        }
    }
}

#[derive(Debug, Default)]
pub struct SemaphoreSet {
    sems: BTreeMap<SemaphoreId, Mutex<Semaphore>>,
}

impl SemaphoreSet {
    pub fn insert(&mut self, key: u32, value: usize) -> bool {
        trace!("Sem Ins : <{:#x}>{}", key, value);
        self.sems
            .insert(SemaphoreId::new(key), Mutex::new(Semaphore::new(value)))
            .is_none()
    }

    pub fn remove(&mut self, key: u32) -> bool {
        trace!("Sem Rem : <{:#x}>", key);
        self.sems.remove(&SemaphoreId::new(key)).is_some()
    }

    pub fn up(&self, key: u32) -> SemaphoreResult {
        let sid = SemaphoreId::new(key);
        if let Some(sem) = self.sems.get(&sid) {
            let mut locked = sem.lock();
            trace!("Sem Up  : <{:#x}>{}", key, locked);
            locked.up()
        } else {
            SemaphoreResult::NoExist
        }
    }

    pub fn down(&self, key: u32, pid: ProcessId) -> SemaphoreResult {
        let sid = SemaphoreId::new(key);
        if let Some(sem) = self.sems.get(&sid) {
            let mut locked = sem.lock();
            trace!("Sem Down: <{:#x}>{}", key, locked);
            locked.down(pid)
        } else {
            SemaphoreResult::NoExist
        }
    }
}

impl core::fmt::Display for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Semaphore({}) {:?}", self.count, self.wait_queue)
    }
}
