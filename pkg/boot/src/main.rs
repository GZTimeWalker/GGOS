#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::*;

#[entry]
fn efi_main(_image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");

    info!("Running UEFI bootloader...");

    system_table.stdout().clear().unwrap();

    let gop = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .expect("Failed to locate graphics output protocol");
    let gop = unsafe { &mut *gop.get() };

    let input = system_table
        .boot_services()
        .locate_protocol::<Input>()
        .expect("failed to get Input");
    let input = unsafe { &mut *input.get() };

    let graphics_mode = gop.current_mode_info();
    let (width, height) = graphics_mode.resolution();

    info!("Current graphics resolution: {}x{}", width, height);

    let fb_addr = gop.frame_buffer().as_mut_ptr() as *mut u32;
    let fb_size = gop.frame_buffer().size();

    info!("Frame buffer address: {:p}", fb_addr);
    info!("Frame buffer size: {}", fb_size);

    for i in 0..5 {
        info!("Waiting for next stage... {}", 5 - i);
        system_table.boot_services().stall(200_000);
    }

    unsafe {
        system_table
            .boot_services()
            .wait_for_event(&mut [input.wait_for_key_event().unsafe_clone()])
            .unwrap();
    }

    clear(fb_addr, width, height);

    system_table
        .stdout()
        .set_color(Color::White, Color::Black)
        .unwrap();

    loop {}
}

fn clear(fb_addr: *mut u32, width: usize, height: usize) {
    for i in 0..width * height {
        unsafe {
            *fb_addr.offset(i as isize) = 0x0;
        }
    }
}
