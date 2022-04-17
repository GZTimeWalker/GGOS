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

pub mod gdt;
pub mod memory;
pub mod process;
pub mod interrupts;

use memory::allocator;
use boot::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    // init serial output
    serial::init();
    println!("[+] Serial Initialized.");

    // init display driver
    display::init(boot_info);
    println!("[+] VGA Display Initialized.");

    // init GDT
    gdt::init();
    println!("[+] GDT Initialized.");

    // init graphic console
    console::init();
    println!("[+] Console Initialized.");

    // init log system
    logger::init();
    info!("Logger Initialized.");

    // init interrupts
    interrupts::init();
    info!("Interrupts Initialized.");

    // init frame allocator
    memory::init(boot_info);

    // init heap allocator
    allocator::init_heap().expect("Heap Initialization Failed.");
    info!("Heap Initialized.");

    // init process manager
    process::init();
    info!("Process Manager Initialized.");

    // init keyboard
    keyboard::init();
    info!("Keyboard Initialized.");

    // init input manager
    input::init();
    info!("Input Initialized.");

    // enable interrupts
    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    // Enable cursor...?
    print_serial!("\x1b[?25h");
}
