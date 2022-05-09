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

pub use interrupt::Syscall;

use boot::BootInfo;
pub use alloc::format;

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
    ata::init();                // init ata
    filesystem::init();         // init filesystem

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    // process::spawn_kernel_thread(
    //     utils::func::clock,
    //     alloc::string::String::from("clock"),
    //     None
    // );

    info!("GGOS initialized.");

    // Enable cursor...?
    print_serial!("\x1b[?25h");
}

pub fn new_test_thread(id: &str) {
    process::spawn_kernel_thread(
        utils::func::test,
        alloc::string::String::from(format!("test_{}", id)),
        Some(process::ProcessData::new().set_env("id", id))
    );
}

pub fn shutdown(boot_info: &'static BootInfo) -> ! {
    info!("GGOS shutting down.");
    unsafe {
        boot_info.system_table.runtime_services().reset(
            boot::ResetType::Shutdown,
            boot::UefiStatus::SUCCESS,
            None,
        );
    }
}
