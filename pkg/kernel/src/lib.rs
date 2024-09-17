#![no_std]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(map_try_insert)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::result_unit_err)]

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate libm;

#[macro_use]
pub mod utils;
use uefi::{runtime::ResetType, Status};
pub use utils::*;

#[macro_use]
pub mod drivers;
pub use drivers::*;

pub mod memory;
pub mod tasks;

pub use tasks::*;

pub mod interrupt;
pub mod proc;

pub use alloc::format;
use boot::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        // set uefi system table
        uefi::table::set_system_table(boot_info.system_table.cast().as_ptr());
    }

    serial::init(); // init serial output
    logger::init(boot_info); // init logger system
    memory::address::init(boot_info); // init memory address
    memory::gdt::init(); // init gdt
    memory::allocator::init(); // init kernel heap allocator
    display::init(boot_info); // init vga display
    console::init(); // init graphic console
    interrupt::init(); // init interrupts
    memory::init(boot_info); // init memory manager
    memory::user::init(); // init user heap allocator
    proc::init(boot_info); // init process manager
    keyboard::init(); // init keyboard
    filesystem::init(); // init filesystem

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    info!("GGOS initialized.");
}

// DEPRECATED: do not spawn kernel thread
// pub fn new_test_thread(id: &str) {
//     process::spawn_kernel_thread(
//         utils::func::test,
//         alloc::string::String::from(format!("test_{}", id)),
//         Some(process::ProcessData::new().set_env("id", id))
//     );
// }

pub fn shutdown() -> ! {
    info!("GGOS shutting down.");
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}
