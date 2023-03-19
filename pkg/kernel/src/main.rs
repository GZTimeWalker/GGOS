#![no_std]
#![no_main]

use ggos::*;
use ggos_kernel as ggos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let mut executor = Executor::new();

    // use executor.spawn() to spawn kernel tasks

    executor.run(spawn_init());
    ggos::shutdown(boot_info);
}

pub fn spawn_init() -> ggos::process::ProcessId {
    let sh_file = ggos::filesystem::try_get_file("/APP/SH", fs::Mode::ReadOnly).unwrap();
    ggos::process::spawn(&sh_file).unwrap()
}
