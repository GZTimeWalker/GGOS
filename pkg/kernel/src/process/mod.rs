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
use alloc::string::String;
use x86_64::{
    registers::control::Cr3,
    structures::{idt::InterruptStackFrame, paging::FrameAllocator},
};

use self::manager::init_PROCESS_MANAGER;

const STACK_BOT: u64 = 0x0000_2000_0000_0000;
const STACK_PAGES: u64 = 512;
const STACK_TOP: u64 = STACK_BOT + STACK_PAGES * 0x1000;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Created,
    Running,
    Ready,
    Blocked,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(u16);

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
        None,
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

pub fn process_exit(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        manager.kill();
        manager.switch_next(regs, sf);
    })
}

pub fn handle(fd: u8) -> Option<Resource> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().current().handle(fd)
    })
}

pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().still_alive(pid)
    })
}

pub fn spawn(file: &File) -> Result<ProcessId, String> {
    let size = file.length();
    let data = {
        let pages = (size as usize + 0x1000 - 1) / 0x1000;
        let allocator = &mut *crate::memory::get_frame_alloc_for_sure();

        let mem_start = allocator.allocate_frame().unwrap().start_address().as_u64();

        trace!("alloc = 0x{:016x}", mem_start);

        for _ in 1..pages {
            let addr = allocator.allocate_frame().unwrap().start_address().as_u64();
            trace!("alloc = 0x{:016x}", addr);
        }

        let mut buf =
            unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };

        fs::read_to_buf(get_volume(), file, &mut buf).expect("Failed to read file");
        &mut buf[..pages * 0x1000]
    };

    let elf = xmas_elf::ElfFile::new(&data).expect("Failed to parse ELF file");

    const STACK_BOT: u64 = 0x0000_2000_0000_0000;
    const STACK_PAGES: u64 = 512;
    const STACK_TOP: u64 = STACK_BOT + STACK_PAGES * 0x1000;

    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();

        let parent = manager.current().pid();
        manager.spawn(
            &elf,
            file.entry.filename(),
            parent,
            Some(ProcessData::new()),
        )
    });
    Ok(pid)
}
