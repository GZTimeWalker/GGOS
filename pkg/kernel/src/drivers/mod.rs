mod uart16550;
mod gop;

pub mod serial;
pub mod console;
pub mod display;
pub mod keyboard;
pub mod input;
pub mod filesystem;
pub mod ata;

pub use input::{get_key, push_key};
