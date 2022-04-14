use alloc::vec::Vec;
use super::*;

once_mutex!(pub PROCESS_MANAGER: ProcessManager);
guard_access_fn! {
    pub get_process_manager(PROCESS_MANAGER: ProcessManager)
}

pub struct ProcessManager {
    current: usize,
    processes: Vec<Process>,
}

impl ProcessManager {
    pub fn new(init: Process) -> Self{
        let current = 0;
        let mut processes = Vec::<Process>::new();
        processes.push(init);
        Self {
            current,
            processes
        }
    }
}
