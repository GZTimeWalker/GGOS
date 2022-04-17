#![no_std]
#![no_main]

use ggos_kernel::{input, print, println};
use ggos_kernel as ggos;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {

    ggos::init(boot_info);

    loop {
        print!(">>> ");
        let something = input::get_line();
        println!("[-] {}", something);
    }
}
