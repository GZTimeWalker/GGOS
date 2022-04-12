mod gop;
mod math;

pub mod colors;
pub mod font;
pub use gop::GOPDisplay;
#[macro_use]
pub mod macros;

pub static HEADER: &str = ">>> GGOS v0.3.1";
