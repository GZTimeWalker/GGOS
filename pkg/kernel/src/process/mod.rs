mod process;
mod manager;
mod scheduler;

// pub mod manager;
use process::*;
use manager::*;
pub use scheduler::*;

use alloc::string::String;
use self::manager::init_PROCESS_MANAGER;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Created,
    Running,
    Ready,
    Blocked,
    Dead
}

/// init process manager
pub fn init() {
    let mut alloc = crate::mem::get_frame_alloc_for_sure();
    // kernel process
    let mut kproc = Process::new( &mut *alloc, 0, String::from("kernel"), 5, 0);
    kproc.resume();
    kproc.set_page_table_with_cr3();
    init_PROCESS_MANAGER(ProcessManager::new(kproc));

    info!("Process Manager Initialized.");
}
