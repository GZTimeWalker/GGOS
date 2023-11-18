use crate::utils::*;
use alloc::format;
use core::convert::TryFrom;
use syscall_def::Syscall;
use x86_64::structures::idt::InterruptStackFrame;

mod service;
use service::*;

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

pub fn dispatcher(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    let args = super::syscall::SyscallArgs::new(
        Syscall::try_from(regs.rax as u16).unwrap(),
        regs.rdi,
        regs.rsi,
        regs.rdx,
    );

    match args.syscall {
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Read => regs.set_rax(sys_read(&args)),
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Write => regs.set_rax(sys_write(&args)),
        // path: &str (arg0 as *const u8, arg1 as len), mode: arg2 as u8 -> fd: u8
        Syscall::Open => regs.set_rax(sys_open(&args)),
        // fd: arg0 as u8 -> success: bool
        Syscall::Close => regs.set_rax(sys_close(&args)),

        // None -> pid: u16
        Syscall::GetPid => regs.set_rax(sys_get_pid() as usize),

        // None -> pid: u16 (diff from parent and child)
        Syscall::VFork => sys_fork(regs, sf),
        // path: &str (arg0 as *const u8, arg1 as len) -> pid: u16
        Syscall::Spawn => regs.set_rax(spawn_process(&args)),
        // pid: arg0 as u16
        Syscall::Exit => exit_process(&args, regs, sf),
        // pid: arg0 as u16 -> status: isize
        Syscall::WaitPid => regs.set_rax(sys_wait_pid(&args)),
        // pid: arg0 as u16
        Syscall::Kill => sys_kill(&args, regs, sf),

        // op: u8, key: u32, val: usize -> ret: any
        Syscall::Sem => sys_sem(&args, regs, sf),
        // None -> time: usize
        Syscall::Time => regs.set_rax(sys_clock() as usize),
        // None
        Syscall::Stat => list_process(),
        // path: &str (arg0 as *const u8, arg1 as len)
        Syscall::ListDir => list_dir(&args),
        // x: arg0 as i32, y: arg1 as i32, color: arg2 as u32
        Syscall::Draw => sys_draw(&args),
        // layout: arg0 as *const Layout -> ptr: *mut u8
        Syscall::Allocate => regs.set_rax(sys_allocate(&args)),
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
