use core::fmt::*;
use crate::console::get_console_for_sure;
use crate::serial::get_serial_for_sure;
use crate::utils::colors;
use x86_64::instructions::interrupts;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::macros::print_internal(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! print_warn {
    ($($arg:tt)*) => ($crate::macros::print_warn_internal(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_serial {
    ($($arg:tt)*) => ($crate::macros::print_serial_internal(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_warn {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print_warn!("{}\n\r", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_serial {
    () => ($crate::print_serial!("\n\r"));
    ($($arg:tt)*) => ($crate::print_serial!("{}\n\r", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        get_console_for_sure().write_fmt(args).unwrap();
        get_serial_for_sure().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn print_warn_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        get_serial_for_sure().write_fmt(args).unwrap();
        let mut console = get_console_for_sure();
        console.set_color(Some(colors::RED), None);
        console.write_fmt(args).unwrap();
        console.set_color(Some(colors::FRONTGROUND), None);
    });
}

#[doc(hidden)]
pub fn print_serial_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        get_serial_for_sure().write_fmt(args).unwrap();
    });
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print_warn!("[!] {}", info);
    loop {}
}
