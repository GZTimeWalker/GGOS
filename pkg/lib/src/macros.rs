use crate::alloc::string::ToString;
use crate::errln;

#[macro_export]
macro_rules! entry {
    ($fn:ident) => {
        #[export_name = "_start"]
        pub extern "C" fn __impl_start() {
            lib::init();
            let ret = $fn();
            lib::sys_exit(ret);
        }
    };
}

#[cfg_attr(not(test), panic_handler)]
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

    errln!(
        "\n\n\rERROR: panicked at {}\n\n\r{}",
        location,
        info.message()
    );

    crate::sys_exit(1);
}
