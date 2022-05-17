mod manager;
mod process;
mod scheduler;

use core::sync::atomic::{AtomicU16, Ordering};

use fs::File;
use manager::*;
use process::*;

pub use process::ProcessData;
pub use scheduler::*;

use crate::{filesystem::get_volume, Registers, Resource};
use alloc::{string::String, vec};
use x86_64::{
    registers::control::{Cr3, Cr2},
    structures::idt::InterruptStackFrame,
};
use x86_64::structures::idt::PageFaultErrorCode;

use self::manager::init_PROCESS_MANAGER;

const STACK_BOT: u64 = 0x0000_2000_0000_0000;
const STACK_PAGES: u64 = 0x100;
const STACK_SIZE: u64 = STACK_PAGES * crate::memory::PAGE_SIZE;
const STACK_START_MASK: u64 = !(STACK_SIZE - 1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Created,
    Running,
    Ready,
    Blocked,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u16);

impl ProcessId {
    pub fn new() -> Self {
        static NEXT_PID: AtomicU16 = AtomicU16::new(0);
        ProcessId(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ProcessId> for u16 {
    fn from(pid: ProcessId) -> Self {
        pid.0
    }
}

/// init process manager
pub fn init() {
    let mut alloc = crate::memory::get_frame_alloc_for_sure();
    // kernel process
    let mut kproc = Process::new(
        &mut *alloc,
        String::from("kernel"),
        ProcessId(0),
        Cr3::read().0,
        None
    );
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

pub fn process_exit(ret: isize, regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        manager.kill(ret);
        manager.switch_next(regs, sf);
    })
}

pub fn wait_pid(pid: ProcessId) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().wait_pid(pid)
    })
}

pub fn handle(fd: u8) -> Option<Resource> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().current().handle(fd)
    })
}

pub fn open(path: &str, mode: u8) -> Option<u8> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().open(path, mode)
    })
}

pub fn close(fd: u8) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().close(fd)
    })
}

pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().still_alive(pid)
    })
}

pub fn current_pid() -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().current_pid()
    })
}

pub fn try_resolve_page_fault(_err_code: PageFaultErrorCode, _sf: &mut InterruptStackFrame) -> Result<(),()> {
    let addr = Cr2::read();
    debug!("Trying to access address: {:?}", addr);

    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager_for_sure();
        debug!("Current process: {:#?}", manager.current());
    });

    Err(())
}

pub fn spawn(file: &File) -> Result<ProcessId, String> {
    let size = file.length();
    let pages = (size as usize + 0x1000 - 1) / 0x1000;
    let mut buf = vec![0u8; (pages * 0x1000) as usize];

    fs::read_to_buf(get_volume(), file, &mut buf).expect("Failed to read file");
    let elf = xmas_elf::ElfFile::new(&buf).expect("Failed to parse ELF file");

    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();

        let parent = manager.current().pid();
        let pid = manager.spawn(
            &elf,
            file.entry.filename(),
            parent,
            Some(ProcessData::new().add_file(file)),
        );

        pid
    });

    Ok(pid)
}

pub fn fork(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        manager.save_current(regs, sf);
        manager.fork();
        manager.switch_next(regs, sf);
    })
}
