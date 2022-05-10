use crate::utils::*;
use core::alloc::Layout;
use x86_64::structures::idt::InterruptStackFrame;

mod service;
use service::*;

#[derive(Clone, Debug)]
pub enum Syscall {
    SpwanProcess = 1,
    ExitProcess = 2,
    Read = 3,
    Write = 4,
    Open = 5,
    Close = 6,
    Stat = 7,
    Clock = 8,
    Draw = 9,
    Allocate = 10,
    Deallocate = 11,
    None = 0xdeadbeef,
}

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

#[allow(unused_variables)]
pub unsafe fn dispatcher(args: SyscallArgs, regs: &mut Registers, sf: &mut InterruptStackFrame) {
    match args.syscall {
        Syscall::SpwanProcess => {}
        Syscall::ExitProcess => exit_process(regs, sf), // todo: handle exit code
        Syscall::Read => match args.arg0 {
            0 => {}
            fd => warn!("SYSCALL: cannot read from fd: {}", fd),
        },
        Syscall::Write => {
            let s = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                args.arg1 as *const u8,
                args.arg2,
            ));
            match args.arg0 {
                1 => print!("{}", s),
                fd => warn!("SYSCALL: cannot write to fd: {}", fd),
            }
        }
        Syscall::Open => {}
        Syscall::Close => {}
        Syscall::Stat => {}
        Syscall::Clock => regs.set_rax(sys_clock() as usize),
        Syscall::Draw => sys_draw(args.arg0, args.arg1, args.arg2),
        Syscall::Allocate => {
            regs.set_rax(sys_allocate((args.arg0 as *const Layout).as_ref().unwrap()) as usize)
        }
        Syscall::Deallocate => sys_deallocate(
            args.arg0 as *mut u8,
            (args.arg1 as *const Layout).as_ref().unwrap(),
        ),
        Syscall::None => {}
    }
}

impl From<usize> for Syscall {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::SpwanProcess,
            2 => Self::ExitProcess,
            3 => Self::Read,
            4 => Self::Write,
            5 => Self::Open,
            6 => Self::Close,
            7 => Self::Stat,
            8 => Self::Clock,
            9 => Self::Draw,
            10 => Self::Allocate,
            11 => Self::Deallocate,
            _ => {
                warn!("Unknown SYSCALL: {}", num);
                Self::None
            }
        }
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
