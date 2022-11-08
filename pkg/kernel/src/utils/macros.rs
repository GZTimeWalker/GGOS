use crate::console::get_console;
use crate::serial::get_serial;
use crate::utils::colors;
use alloc::string::ToString;
use core::fmt::*;
use x86_64::instructions::interrupts;

/// Use spin mutex to control variable access
#[macro_export]
macro_rules! guard_access_fn {
    ($(#[$meta:meta])* $v:vis $fn:ident ($mutex:path : $ty:ty)) => {
        paste::item! {

            $(#[$meta])*
            #[allow(non_snake_case, dead_code)]
            $v fn $fn<'a>() -> Option<spin::MutexGuard<'a, $ty>> {
                $mutex.get().and_then(spin::Mutex::try_lock)
            }

            $(#[$meta])*
            #[allow(non_snake_case, dead_code)]
            $v fn [< $fn _for_sure >]<'a>() -> spin::MutexGuard<'a, $ty> {
                $mutex.get().and_then(spin::Mutex::try_lock).expect(
                    stringify!($mutex has not been initialized or lockable)
                )
            }
        }
    };
}

#[macro_export]
macro_rules! once_mutex {
    ($i:vis $v:ident: $t:ty) => {
        $i static $v: spin::Once<spin::Mutex<$t>> = spin::Once::new();

        paste::item! {
            #[allow(non_snake_case)]
            $i fn [<init_ $v>]([<val_ $v>]: $t) {
                $v.call_once(|| spin::Mutex::new([<val_ $v>]));
            }
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::utils::print_internal(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! print_warn {
    ($($arg:tt)*) => ($crate::utils::print_warn_internal(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_serial {
    ($($arg:tt)*) => ($crate::utils::print_serial_internal(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_warn {
    () => ($crate::print_warn!("\n\r"));
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
        if let Some(mut serial) = get_serial() {
            serial.write_fmt(args).unwrap();
        }

        if let Some(mut console) = get_console() {
            console.write_fmt(args).unwrap();
        }
    });
}

#[doc(hidden)]
pub fn print_warn_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        if let Some(mut serial) = get_serial() {
            serial.write_fmt(args).unwrap();
        }

        if let Some(mut console) = get_console() {
            console.set_color(Some(colors::RED), None);
            console.write_fmt(args).unwrap();
            console.set_color(Some(colors::FRONTGROUND), None);
        }
    });
}

#[doc(hidden)]
pub fn print_serial_internal(args: Arguments) {
    interrupts::without_interrupts(|| {
        if let Some(mut serial) = get_serial() {
            serial.write_fmt(args).unwrap();
        }
    });
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let location = if let Some(location) = info.location() {
        alloc::format!(
            "{}@{}:{}",
            location.file(),
            location.line(),
            location.column()
        )
    } else {
        "Unknown location".to_string()
    };
    let msg = if let Some(msg) = info.message() {
        alloc::format!("{:#?}", msg)
    } else {
        "No more message...".to_string()
    };
    error!("\n\n\rERROR: panicked at {}\n\n\r{}", location, msg);
    loop {}
}
