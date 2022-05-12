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
        Syscall::SpawnProcess => regs.set_rax(spawn_process(&args)),
        Syscall::ExitProcess => exit_process(regs, sf),
        Syscall::Read => regs.set_rax(sys_read(&args)),
        Syscall::Write => sys_write(&args),
        Syscall::Open => {}
        Syscall::Close => {}
        Syscall::Stat => list_process(),
        Syscall::Time => regs.set_rax(sys_clock() as usize),
        Syscall::DirectoryList => list_dir(&args),
        Syscall::Allocate => regs.set_rax(sys_allocate(&args)),
        Syscall::Deallocate => sys_deallocate(&args),
        Syscall::Draw => sys_draw(&args),
        Syscall::None => {}
    }
    // debug!("syscall finished.");
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
