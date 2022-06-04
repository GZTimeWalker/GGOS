#![no_std]
#![no_main]

use ggos::*;
use ggos_kernel as ggos;

extern crate alloc;

#[macro_use]
extern crate log;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let mut executor = Executor::new();

    // TODO: use executor.spawn() to spawn kernel tasks

    ggos::process::print_process_list();
    debug!("Testing stack auto grow...");
    stack_test();
    ggos::process::print_process_list();

    executor.run(spawn_init());
    ggos::shutdown(boot_info);
}

pub fn stack_test() {
    let mut stack = [0u64; 0x1000];

    for i in 0..stack.len() {
        stack[i] = i as u64;
    }

    for i in 0..stack.len() / 512 {
        assert!(stack[i * 512] == i as u64 * 512);
    }
}

pub fn spawn_init() -> ggos::process::ProcessId {
    let sh_file = ggos::filesystem::try_get_file("/APP/SH", fs::Mode::ReadOnly).unwrap();
    ggos::process::spawn(&sh_file).unwrap()
}
