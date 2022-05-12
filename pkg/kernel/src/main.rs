#![no_std]
#![no_main]

use ggos::*;
use ggos_kernel as ggos;

extern crate alloc;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let mut executor = Executor::new();

    let sh_file = ggos::filesystem::try_get_file("/APP/SH").unwrap();
    let pid = ggos::process::spawn(&sh_file).unwrap();

    executor.run(pid);

    ggos::shutdown(boot_info);
}
