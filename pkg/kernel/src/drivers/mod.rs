mod gop;
mod uart16550;

pub mod ata;
pub mod console;
pub mod display;
pub mod filesystem;
pub mod input;
pub mod keyboard;
pub mod serial;

pub use filesystem::get_rootfs;
pub use input::{get_key, push_key};
