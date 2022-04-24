#![no_std]
#![allow(dead_code)]
#![feature(asm_sym)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(panic_info_message)]

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod utils;
pub use utils::*;

#[macro_use]
pub mod drivers;
pub use drivers::*;

pub mod memory;
use memory::gdt;
use memory::allocator;

pub mod process;
pub mod interrupt;

use boot::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    serial::init();             // init serial output
    logger::init();             // init logger system
    gdt::init();                // init gdt
    display::init(boot_info);   // init vga display
    console::init();            // init graphic console
    clock::init(boot_info);     // init clock (uefi service)
    interrupt::init();          // init interrupts
    memory::init(boot_info);    // init memory manager
    allocator::init();          // init heap allocator
    process::init();            // init process manager
    keyboard::init();           // init keyboard
    input::init();              // init input manager

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    process::spawn_kernel_thread(
        utils::draw::clock,
        alloc::string::String::from("clock"),
        5
    );

    info!("GGOS initialized.");

    // Enable cursor...?
    print_serial!("\x1b[?25h");
}
