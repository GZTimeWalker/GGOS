#![no_std]
#![no_main]
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
mod utils;
#[macro_use]
mod drivers;
mod gdt;
mod interrupts;
mod memory;
mod process;

use drivers::*;
use memory::allocator;
use boot::BootInfo;
use x86_64::VirtAddr;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();

    unsafe { drivers::serial::init(); }

    println!("{}",utils::get_ascii_header());
    println!("[+] Serial Initialized.");

    // init display driver
    let graphic_info = &boot_info.graphic_info;
    display::init(graphic_info);
    display::get_display_for_sure().clear(Some(utils::colors::BACKGROUND), 0);

    // init graphic console
    console::init();
    println!("[+] Console Initialized.");

    // init log system
    utils::logger::init();
    info!("Logger Initialized.");

    // init interrupts
    unsafe { interrupts::init(); }
    info!("Interrupts Initialized.");

    // init frame allocator
    unsafe {
        memory::init(
            VirtAddr::new_truncate(memory::PHYSICAL_OFFSET as u64),
            &boot_info.memory_map);
    }

    allocator::init_heap(
        &mut *memory::get_page_table_for_sure(),
        &mut *memory::get_frame_alloc_for_sure(),
        ).expect("Heap Initialization Failed.");
    info!("Heap Initialized.");

    process::init();
    info!("Process Manager Initialized.");

    drivers::keyboard::init();
    info!("Keyboard Initialized.");

    drivers::input::init();
    info!("Input Initialized.");

    // enable interrupts
    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    unsafe {
        *(0xdead0000beef as *mut u64) = 42;
    };

    loop {
        print!(">>> ");
        let something = drivers::input::get_line();
        println!("[-] {}", something);
    }
}
