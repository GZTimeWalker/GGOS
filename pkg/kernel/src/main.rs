#![no_std]
#![no_main]

use ggos::utils;
use ggos_kernel::{input, print, println};
use ggos_kernel as ggos;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {

    ggos::init(boot_info);

    let mut test_num = 0;

    loop {
        print!("[>] ");
        let line = input::get_line();
        match line.trim() {
            "exit" => break,
            "ps" => {
                ggos::process::print_process_list();
            },
            "test" => {
                todo!()
            },
            _ => println!("[=] {}", line),
        }
    }

    ggos::shutdown(boot_info);
}
