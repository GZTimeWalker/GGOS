use crate::utils::*;
use x86_64::structures::idt::InterruptStackFrame;
use num_enum::TryFromPrimitive;
use core::convert::TryFrom;

mod service;
use service::*;

#[repr(u8)]
#[derive(Clone, Debug, TryFromPrimitive)]
pub enum Syscall {
    SpawnProcess = 1,
    ExitProcess = 2,
    Read = 3,
    Write = 4,
    Open = 5,
    Close = 6,
    Stat = 7,
    Time = 8,
    DirectoryList = 9,
    Allocate = 10,
    Deallocate = 11,
    Draw = 12,
    WaitPid = 13,
    #[num_enum(default)]
    None = 255,
}

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

pub fn dispatcher(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    let args = super::syscall::SyscallArgs::new(
        Syscall::try_from(regs.rax as u8).unwrap(),
        regs.rdi,
        regs.rsi,
        regs.rdx
    );

    match args.syscall {
        // path: &str (arg0 as *const u8, arg1 as len) -> pid: u16
        Syscall::SpawnProcess   => regs.set_rax(spawn_process(&args)),
        // pid: arg0 as u16
        Syscall::ExitProcess    => exit_process(&args, regs, sf),
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Read           => regs.set_rax(sys_read(&args)),
        // fd: arg0 as u8, buf: &[u8] (arg1 as *const u8, arg2 as len)
        Syscall::Write          => regs.set_rax(sys_write(&args)),
        // path: &str (arg0 as *const u8, arg1 as len), mode: arg2 as u8 -> fd: u8
        Syscall::Open           => regs.set_rax(sys_open(&args)),
        // fd: arg0 as u8 -> success: bool
        Syscall::Close          => regs.set_rax(sys_close(&args)),
        // None
        Syscall::Stat           => list_process(),
        // None -> time: usize
        Syscall::Time           => regs.set_rax(sys_clock() as usize),
        // path: &str (arg0 as *const u8, arg1 as len)
        Syscall::DirectoryList  => list_dir(&args),
        // layout: arg0 as *const Layout -> ptr: *mut u8
        Syscall::Allocate       => regs.set_rax(sys_allocate(&args)),
        // ptr: arg0 as *mut u8
        Syscall::Deallocate     => sys_deallocate(&args),
        // x: arg0 as i32, y: arg1 as i32, color: arg2 as u32
        Syscall::Draw           => sys_draw(&args),
        // pid: arg0 as u16 -> status: isize
        Syscall::WaitPid        => regs.set_rax(sys_wait_pid(&args)),
        // None
        Syscall::None           => {}
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
            "SYSCALL: {:?} (0x{:016x}, 0x{:016x}, 0x{:016x})",
            self.syscall, self.arg0, self.arg1, self.arg2
        )
    }
}
