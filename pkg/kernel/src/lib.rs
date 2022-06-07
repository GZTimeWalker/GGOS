#![no_std]
#![allow(dead_code)]
#![feature(asm_sym)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(panic_info_message)]
#![feature(map_try_insert)]

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
pub mod tasks;

pub use tasks::*;
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
    process::init(boot_info);   // init process manager
    keyboard::init();           // init keyboard
    input::init();              // init input
    ata::init();                // init ata
    filesystem::init();         // init filesystem

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    info!("GGOS initialized.");

    print_serial!("\x1b[?25h");
}

// DEPRECATED: do not spawn kernel thread
// pub fn new_test_thread(id: &str) {
//     process::spawn_kernel_thread(
//         utils::func::test,
//         alloc::string::String::from(format!("test_{}", id)),
//         Some(process::ProcessData::new().set_env("id", id))
//     );
// }

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
