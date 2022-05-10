use crate::{Syscall, errln};
use crate::alloc::string::ToString;
use core::arch::asm;

#[doc(hidden)]
pub fn syscall0(n: Syscall) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
pub fn syscall1(n: Syscall, arg0: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
pub fn syscall2(n: Syscall, arg0: usize, arg1: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0, in("rsi") arg1,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
pub fn syscall3(n: Syscall, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0, in("rsi") arg1, in("rdx") arg2,
            lateout("rax") ret
        );
    }
    ret
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => {
        $crate::macros::syscall0($n)
    };
    ($n:expr, $a1:expr) => {
        $crate::macros::syscall1($n, $a1 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr) => {
        $crate::macros::syscall2($n, $a1 as usize, $a2 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::macros::syscall3($n, $a1 as usize, $a2 as usize, $a3 as usize)
    };
}

#[macro_export]
macro_rules! entry {
    ($fn:ident) => {
        #[export_name = "_start"]
        pub extern "C" fn __impl_start() {
            $fn();
        }
    };
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let location = if let Some(location) = info.location() {
        alloc::format!("{}@{}:{}", location.file(), location.line(), location.column())
    } else {
        "Unknown location".to_string()
    };
    let msg = if let Some(msg) = info.message() {
        alloc::format!("{:#?}", msg)
    } else {
        "No more message...".to_string()
    };
    errln!("\n\n\rERROR: panicked at {}\n\n\r{}", location, msg);
    loop {}
}
