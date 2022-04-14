mod gop;
mod math;
#[macro_use]
mod macros;

pub mod draw;
pub mod font;
pub mod colors;
pub mod logger;
pub use gop::GOPDisplay;
pub use macros::*;

pub const fn get_ascii_header() -> &'static str {
    concat!("
 _______  _______  _______  _______
|       ||       ||       ||       |
|    ___||    ___||   _   ||  _____|
|   | __ |   | __ |  | |  || |_____
|   ||  ||   ||  ||  |_|  ||_____  |
|   |_| ||   |_| ||       | _____| |
|_______||_______||_______||_______|
                                v", env!("CARGO_PKG_VERSION"), " by GZTime")
}

pub const fn get_header() -> &'static str {
    concat!(">>> GGOS v", env!("CARGO_PKG_VERSION"))
}

#[repr(align(8), C)]
#[derive(Debug, Clone, Default)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rbp: usize,
}
