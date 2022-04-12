mod gop;
mod math;

pub mod draw;
pub mod colors;
pub mod font;
pub use gop::GOPDisplay;
#[macro_use]
pub mod macros;

pub static HEADER: &str = ">>> GGOS v0.3.1";

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
