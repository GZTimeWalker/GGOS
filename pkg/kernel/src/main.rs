#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]

use boot::BootInfo;
// use core::arch::asm;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;
#[macro_use]
mod console;

mod display;
mod gdt;
mod interrupts;
mod logger;
mod memory;
mod utils;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();

    let graphic_info = &boot_info.graphic_info;
    display::initialize(graphic_info);

    display::get_display_for_sure().clear(Some(utils::colors::BACKGROUND), 0);

    console::initialize();
    println!("[+] Console Initialized.");

    logger::initialize();
    info!("Logger Initialized.");

    unsafe {
        interrupts::init();
    }
    info!("Interrupts Initialized.");

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    let alpha = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    for _ in 0..100 {
        print!("{}", alpha);
        for _ in 0..50_0000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!();
    trace!("Trace?");
    debug!("Debug Test.");
    warn!("Warning Test.");
    error!("ERROR!!!");
    print!("Hello, world!");

    loop {}
}
