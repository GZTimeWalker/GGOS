#![allow(dead_code)]

mod process;

// pub mod manager;
pub use process::*;

use alloc::string::String;
use alloc::vec::Vec;
use x86_64::registers::control::Cr3;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Creating,
    Running,
    Ready,
    Blocked,
    Dead
}

/// init process system, allocator required
pub fn init() {
    init_PROCESSES(Vec::new());
    let mut alloc = crate::memory::get_frame_alloc_for_sure();
    let mut list = get_processes_for_sure();
    // kernel process
    let mut kproc = Process::new(
        &mut *alloc,
        0,
        String::from("kernel"),
        10);
    kproc.status = ProgramStatus::Running;
    kproc.page_table_addr = Cr3::read();
    list.push(kproc);
}
