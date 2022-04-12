#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_error_handler)]

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
// use core::arch::asm;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();

    // init serial output driver
    unsafe {
        serial::initialize();
    }

    // init display driver
    let graphic_info = &boot_info.graphic_info;
    display::initialize(graphic_info);
    display::get_display_for_sure().clear(Some(utils::colors::BACKGROUND), 0);

    // init graphic console
    console::initialize();
    println!("[+] Console Initialized.");

    // init log system
    utils::logger::initialize();
    info!("Logger Initialized.");

    // init interrupts
    unsafe {
        interrupts::init();
    }
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

    trace!("Trace?");
    debug!("Debug Test.");
    warn!("Warning Test.");
    error!("ERROR!!!");

    for i in 0..10 {
        print!("[>] Waiting [{:>2}] ", i);
        for _ in 0..50 {
            for _ in 0..200_0000 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
            print!(">");
        }
        println!();
    }

    // enable interrupts
    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    loop {}
}
