mod process;
mod manager;
mod scheduler;

use process::*;
use manager::*;

pub use scheduler::*;
pub use process::ProcessData;

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
    let mut alloc = crate::memory::get_frame_alloc_for_sure();
    // kernel process
    let mut kproc = Process::new( &mut *alloc, 0, String::from("kernel"), 0, None);
    kproc.resume();
    kproc.set_page_table_with_cr3();
    init_PROCESS_MANAGER(ProcessManager::new(kproc));

    info!("Process Manager Initialized.");
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().current().env(key)
    })
}
