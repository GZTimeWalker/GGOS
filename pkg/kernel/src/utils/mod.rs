mod uefi;

#[macro_use]
mod macros;
#[macro_use]
mod regs;

pub mod clock;
pub mod colors;
pub mod font;
pub mod func;
pub mod logger;
pub mod resource;

pub use macros::*;
pub use regs::*;
pub use resource::Resource;
use x86_64::instructions::interrupts;

pub const fn get_ascii_header() -> &'static str {
    concat!(
        "
 _______  _______  _______  _______
|       ||       ||       ||       |
|    ___||    ___||   _   ||  _____|
|   | __ |   | __ |  | |  || |_____
|   ||  ||   ||  ||  |_|  ||_____  |
|   |_| ||   |_| ||       | _____| |
|_______||_______||_______||_______|
                                v",
        env!("CARGO_PKG_VERSION"),
        " by GZTime"
    )
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

const SHORT_UNITS: [&str; 4] = ["B", "K", "M", "G"];
const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

pub fn humanized_size(size: u64) -> (f32, &'static str) {
    humanized_size_impl(size, false)
}

pub fn humanized_size_short(size: u64) -> (f32, &'static str) {
    humanized_size_impl(size, true)
}

#[inline]
pub fn humanized_size_impl(size: u64, short: bool) -> (f32, &'static str) {
    let bytes = size as f32;

    let units = if short { &SHORT_UNITS } else { &UNITS };

    let mut unit = 0;
    let mut bytes = bytes;

    while bytes >= 1024f32 && unit < units.len() {
        bytes /= 1024f32;
        unit += 1;
    }

    (bytes, units[unit])
}
