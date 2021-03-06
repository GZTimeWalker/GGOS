mod math;
mod uefi;

#[macro_use]
mod macros;
#[macro_use]
mod regs;

pub mod func;
pub mod font;
pub mod colors;
pub mod logger;
pub mod clock;
pub mod resource;

pub use resource::Resource;
pub use macros::*;
pub use regs::*;
use x86_64::instructions::interrupts;

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

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}
