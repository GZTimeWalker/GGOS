mod context;
mod data;
mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod sync;

use alloc::sync::Arc;
use alloc::vec::Vec;
use manager::*;
use paging::*;
use process::*;
use storage::FileSystem;
use sync::*;

pub use context::ProcessContext;
pub use data::ProcessData;
pub use pid::ProcessId;
use xmas_elf::ElfFile;

use crate::get_rootfs;
use alloc::string::{String, ToString};
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::{registers::control::Cr2, structures::idt::InterruptStackFrame, VirtAddr};

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
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_BOT: u64 = KSTACK_MAX - STACK_MAX_SIZE;
pub const KSTACK_DEF_PAGE: u64 = 8;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init(boot_info: &'static boot::BootInfo) {
    let kproc_data = ProcessData::new()
        .set_stack(KSTACK_INIT_BOT, KSTACK_DEF_PAGE)
        .set_kernel_code(&boot_info.kernel_pages);

    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    let kproc = Process::new(
        String::from("kernel"),
        None,
        PageTableContext::new(),
        Some(kproc_data),
    );

    kproc.write().resume();
    manager::init(kproc);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = manager.save_current(context);
        manager.push_ready(pid);
        manager.switch_next(context);
    });
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        manager.kill_self(ret);
        manager.switch_next(context);
    })
}

pub fn wait_pid(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.wait_pid(pid) {
            context.set_rax(ret as usize);
        } else {
            manager.save_current(context);
            manager.current().write().block();
            manager.switch_next(context);
        }
    })
}

pub fn read(fd: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().read(fd, buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().write(fd, buf))
}

pub fn open(path: &str, mode: u8) -> Option<u8> {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().open(path, mode))
}

pub fn close(fd: u8) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().close(fd))
}

pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().get_ret(pid).is_none()
    })
}

pub fn current_pid() -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(processor::current_pid)
}

pub fn kill(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if pid == processor::current_pid() {
            manager.kill_self(0xdead);
            manager.switch_next(context);
        } else {
            manager.kill(pid, 0xdead);
        }
    })
}

pub fn new_sem(key: u32, value: usize) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().current().write().new_sem(key, value) as isize
    })
}

pub fn remove_sem(key: u32) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().current().write().remove_sem(key) as isize
    })
}

pub fn sem_up(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().sem_up(key);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NoExist => context.set_rax(1),
            SemaphoreResult::WakeUp(pid) => manager.wake_up(pid),
            _ => unreachable!(),
        }
    })
}

pub fn sem_down(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = processor::current_pid();
        let ret = manager.current().write().sem_down(key, pid);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NoExist => context.set_rax(1),
            SemaphoreResult::Block(pid) => {
                manager.save_current(context);
                manager.block(pid);
                manager.switch_next(context);
            }
            _ => unreachable!(),
        }
    })
}

pub fn try_resolve_page_fault(
    _err_code: PageFaultErrorCode,
    _sf: &mut InterruptStackFrame,
) -> Result<(), ()> {
    let addr = Cr2::read();
    debug!("Trying to access address: {:?}", addr);

    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        debug!("Current process: {:#?}", manager.current());
    });

    Err(())
}

pub fn fs_spawn(path: &str) -> Option<ProcessId> {
    let handle = get_rootfs().open_file(path);

    if let Err(e) = handle {
        warn!("fs_spawn: file error: {}, err: {:?}", path, e);
        return None;
    }

    let mut handle = handle.unwrap();

    let mut file_buffer = Vec::new();

    if let Err(e) = handle.read_all(&mut file_buffer) {
        warn!("fs_spawn: failed to read file: {}, err: {:?}", path, e);
        return None;
    }

    match spawn(handle.meta.name, file_buffer) {
        Ok(pid) => Some(pid),
        Err(e) => {
            warn!("fs_spawn: failed to spawn process: {}, {}", path, e);
            None
        }
    }
}

pub fn spawn(name: String, file_buffer: Vec<u8>) -> Result<ProcessId, String> {
    let elf = xmas_elf::ElfFile::new(&file_buffer).map_err(|e| e.to_string())?;

    let pid = elf_spawn(name, &elf)?;

    Ok(pid)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Result<ProcessId, String> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();

        let parent = Arc::downgrade(&manager.current());

        let pid = manager.spawn(elf, name, Some(parent), None);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Ok(pid)
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let parent = manager.save_current(context);
        manager.fork();
        manager.push_ready(parent);
        manager.switch_next(context);
    })
}

pub fn current_proc_info() {
    debug!("{:#?}", get_process_manager().current())
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> Result<(), ()> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
