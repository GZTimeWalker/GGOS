mod manager;
mod sync;

#[allow(clippy::module_inception)]
mod process;

use core::sync::atomic::{AtomicU16, Ordering};

use alloc::collections::btree_map::Entry;
use fs::File;
use manager::*;
use process::*;
use sync::*;

pub use process::ProcessData;

use crate::{filesystem::get_volume, Registers, Resource};
use alloc::{collections::BTreeMap, string::String, vec};
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::{
    registers::control::{Cr2, Cr3},
    structures::idt::InterruptStackFrame,
    VirtAddr,
};

use self::manager::init_PROCESS_MANAGER;
use self::sync::init_SEMAPHORES;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;
// stack max addr, every thread has a stack space
// from 0x????_????_0000_0000 to 0x????_????_ffff_ffff
// 0x100000000 bytes -> 4GiB
// allow 0x2000 (4096) threads run as a time
// 0x????_2000_????_???? -> 0x????_3fff_????_????
// init alloc stack has size of 0x2000 (2 frames)
// every time we meet a page fault, we alloc more frames
pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * crate::memory::PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_BOT: u64 = STACK_MAX - STACK_MAX_SIZE;
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * crate::memory::PAGE_SIZE;
pub const STACT_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 1;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_BOT: u64 = KSTACK_MAX - STACK_MAX_SIZE;
pub const KSTACK_DEF_PAGE: u64 = 8;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 1;

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

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
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
pub fn init(boot_info: &'static boot::BootInfo) {
    let mut alloc = crate::memory::get_frame_alloc_for_sure();
    let kproc_data = ProcessData::new()
        .set_stack(KSTACK_INIT_BOT, KSTACK_DEF_PAGE)
        .set_kernel_code(&boot_info.kernel_pages);
    trace!("Init process data: {:#?}", kproc_data);
    // kernel process
    let mut kproc = Process::new(
        &mut alloc,
        String::from("kernel"),
        ProcessId::new(),
        Cr3::read().0,
        Some(kproc_data),
    );
    kproc.resume();
    kproc.set_page_table_with_cr3();
    init_PROCESS_MANAGER(ProcessManager::new(kproc));
    init_SEMAPHORES(BTreeMap::new());
    info!("Process Manager Initialized.");
}

pub fn switch(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        manager.save_current(regs, sf);
        manager.switch_next(regs, sf);
    });
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
        manager.kill_self(ret);
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

pub fn kill(pid: ProcessId, regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        if pid == manager.current_pid() {
            manager.kill_self(0xdead);
            manager.switch_next(regs, sf);
        } else {
            manager.kill(pid, 0xdead);
        }
    })
}

pub fn new_sem(key: u32, value: usize) -> isize {
    if let Some(mut sems) = get_sem_manager() {
        let sid = SemaphoreId::new(key);
        if let Entry::Vacant(e) = sems.entry(sid) {
            e.insert(Semaphore::new(value));
            return 0;
        }
    }
    1
}

pub fn sem_up(key: u32) -> isize {
    if let Some(mut sems) = get_sem_manager() {
        let sid = SemaphoreId::new(key);
        if let Some(sem) = sems.get_mut(&sid) {
            // debug!("<{:#x}>{}", key, sem);
            if let Some(pid) = sem.up() {
                trace!("Semaphore #{:#x} up -> unblock process: #{}", key, pid);
                let mut manager = get_process_manager_for_sure();
                manager.unblock(pid);
            }
            return 0;
        }
    }
    1
}

pub fn sem_down(key: u32, regs: &mut Registers, sf: &mut InterruptStackFrame) {
    if let Some(mut sems) = get_sem_manager() {
        let sid = SemaphoreId::new(key);
        if let Some(sem) = sems.get_mut(&sid) {
            // debug!("<{:#x}>{}", key, sem);
            let mut manager = get_process_manager_for_sure();
            let pid = manager.current_pid();
            if let Err(()) = sem.down(pid) {
                trace!("Semaphore #{:#x} down -> block process: #{}", key, pid);
                regs.set_rax(0);
                manager.save_current(regs, sf);
                manager.block(pid);
                manager.switch_next(regs, sf);
            } else {
                regs.set_rax(0);
            }
        } else {
            regs.set_rax(1);
        }
    } else {
        regs.set_rax(1);
    }
}

pub fn remove_sem(key: u32) -> isize {
    if let Some(mut sems) = get_sem_manager() {
        let key = SemaphoreId::new(key);
        if sems.remove(&key).is_some() {
            0
        } else {
            1
        }
    } else {
        1
    }
}

pub fn try_resolve_page_fault(
    _err_code: PageFaultErrorCode,
    _sf: &mut InterruptStackFrame,
) -> Result<(), ()> {
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
    let mut buf = vec![0u8; pages * 0x1000];

    fs::read_to_buf(get_volume(), file, &mut buf).map_err(|_| "Failed to read file")?;

    let elf = xmas_elf::ElfFile::new(&buf).map_err(|_| "Invalid ELF file")?;

    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();

        let parent = manager.current().pid();
        let pid = manager.spawn(
            &elf,
            file.entry.filename(),
            parent,
            Some(ProcessData::new().add_file(file)),
        );
        debug!(
            "Spawned process: {}#{}",
            file.entry.filename().to_lowercase(),
            pid
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

pub fn force_show_info() {
    unsafe {
        manager::PROCESS_MANAGER.get().unwrap().force_unlock();
    }

    debug!("{:#?}", get_process_manager_for_sure().current())
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> Result<(), ()> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager_for_sure().handle_page_fault(addr, err_code)
    })
}
