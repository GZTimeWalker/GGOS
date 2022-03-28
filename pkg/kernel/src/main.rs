#![no_std]
#![no_main]

use core::arch::asm;
use boot::BootInfo;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;
#[macro_use]
mod console;

mod utils;
mod display;
mod logger;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static BootInfo) -> ! {

    let graphic_info = &boot_info.graphic_info;
    display::initialize(graphic_info);
    display::get_display_for_sure().clear(None);

    console::initialize();
    println!("[+] Console Initialized.");

    logger::initialize();
    info!("Logger Initialized.");

    warn!("Warn Testing...");

    for i in 0..5 {
        info!("Testing...{}", i);
        for _ in 0..5000000 {
            unsafe {
                asm!("nop");
            }
        }
    }

    panic!("Exit!!");

    loop{}
}
