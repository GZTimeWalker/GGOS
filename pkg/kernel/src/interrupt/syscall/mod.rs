use crate::{memory::gdt, proc::ProcessContext};
use alloc::format;
use syscall_def::Syscall;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

mod service;
use super::consts;
use service::*;

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[consts::Interrupts::Syscall as u8]
        .set_handler_fn(syscall_handler)
        .set_stack_index(gdt::SYSCALL_IST_INDEX)
        .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
}

pub extern "C" fn syscall(mut context: ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        super::syscall::dispatcher(&mut context);
    });
}

as_handler!(syscall);

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

pub fn dispatcher(context: &mut ProcessContext) {
    let args = super::syscall::SyscallArgs::new(
        Syscall::from(context.regs.rax as u16),
        context.regs.rdi,
        context.regs.rsi,
        context.regs.rdx,
    );

    match args.syscall {
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Read => context.set_rax(sys_read(&args)),
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Write => context.set_rax(sys_write(&args)),
        // path: &str (arg0 as *const u8, arg1 as len), mode: arg2 as u8 -> fd: u8
        Syscall::Open => context.set_rax(sys_open(&args)),
        // fd: arg0 as u8 -> success: bool
        Syscall::Close => context.set_rax(sys_close(&args)),

        // None -> pid: u16
        Syscall::GetPid => context.set_rax(sys_get_pid() as usize),

        // None -> pid: u16 (diff from parent and child)
        Syscall::VFork => sys_fork(context),
        // path: &str (arg0 as *const u8, arg1 as len) -> pid: u16
        Syscall::Spawn => context.set_rax(spawn_process(&args)),
        // pid: arg0 as u16
        Syscall::Exit => exit_process(&args, context),
        // pid: arg0 as u16 -> status: isize
        Syscall::WaitPid => context.set_rax(sys_wait_pid(&args)),
        // pid: arg0 as u16
        Syscall::Kill => sys_kill(&args, context),

        // op: u8, key: u32, val: usize -> ret: any
        Syscall::Sem => sys_sem(&args, context),
        // None -> time: usize
        Syscall::Time => context.set_rax(sys_clock() as usize),
        // x: arg0 as i32, y: arg1 as i32, color: arg2 as u32
        Syscall::Draw => sys_draw(&args),
        // None
        Syscall::Stat => list_process(),
        // path: &str (arg0 as *const u8, arg1 as len)
        Syscall::ListDir => list_dir(&args),

        // layout: arg0 as *const Layout -> ptr: *mut u8
        Syscall::Allocate => context.set_rax(sys_allocate(&args)),
        // ptr: arg0 as *mut u8
        Syscall::Deallocate => sys_deallocate(&args),
        // None
        Syscall::None => {}
    }
}

impl SyscallArgs {
    pub fn new(syscall: Syscall, arg0: usize, arg1: usize, arg2: usize) -> Self {
        Self {
            syscall,
            arg0,
            arg1,
            arg2,
        }
    }
}

impl core::fmt::Display for SyscallArgs {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "SYSCALL: {:<10} (0x{:016x}, 0x{:016x}, 0x{:016x})",
            format!("{:?}", self.syscall),
            self.arg0,
            self.arg1,
            self.arg2
        )
    }
}
