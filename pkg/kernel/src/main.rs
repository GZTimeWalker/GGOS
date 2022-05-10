#![no_std]
#![no_main]

use ggos_kernel as ggos;

extern crate alloc;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let sh_file = ggos::filesystem::try_get_file("/APP/SH").unwrap();
    let pid = ggos::process::spawn(&sh_file).unwrap();

    while ggos::process::still_alive(pid) {
        unsafe {
            core::arch::asm!("hlt");
        }
    }

    ggos::shutdown(boot_info);
}
