#![no_std]
#![allow(dead_code)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
pub mod macros;

#[macro_use]
pub mod io;
mod syscall;
mod utils;

#[macro_use]
mod sync;
pub mod allocator;

use core::fmt::*;
pub use alloc::*;

pub use io::*;
pub use syscall::*;
pub use chrono::*;
pub use utils::*;
pub use sync::*;

#[derive(Clone, Debug)]
pub enum Syscall {
    Spawn = 1,
    Exit = 2,
    Read = 3,
    Write = 4,
    Open = 5,
    Close = 6,
    Stat = 7,
    Time = 8,
    ListDir = 9,
    Allocate = 10,
    Deallocate = 11,
    Draw = 12,
    WaitPid = 13,
    GetPid = 14,
    Fork = 15,
    Kill = 16,
    Sem = 17,
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => ($crate::_err(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! errln {
    () => ($crate::err!("\n"));
    ($($arg:tt)*) => ($crate::err!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    stdout().write(format!("{}", args).as_str());
}

#[doc(hidden)]
pub fn _err(args: Arguments) {
    stderr().write(format!("{}", args).as_str());
}
