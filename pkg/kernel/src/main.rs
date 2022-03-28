#![no_std]
#![no_main]

use core::arch::asm;

use boot::BootInfo;


#[macro_use]
mod macros;

mod utils;
mod display;
mod console;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static BootInfo) -> ! {

    let graphic_info = &boot_info.graphic_info;
    display::initialize(graphic_info);
    display::get_display_for_sure().clear(None);

    console::initialize();
    println!("Console Initialized.");

    println!("CharSetTest: !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz~?");

    for i in 0..100 {
        println!("Testing...{}", i);
        for _ in 0..5000_0000 {
            unsafe {
                asm!("nop");
            }
        }
    }

    panic!("Exit!!");
}
