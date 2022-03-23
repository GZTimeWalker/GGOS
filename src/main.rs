#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;
extern crate rlibc;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::*;

#[entry]
fn efi_main(_image: uefi::Handle, mut st: SystemTable<Boot>) -> Status {

    uefi_services::init(&mut st).expect_success("[!] Failed to initialize utilities");

    info!("[+] Running UEFI bootloader...");

    let bs = st.boot_services();
    let gop = bs
        .locate_protocol::<GraphicsOutput>()
        .expect_success("[!] Failed to locate GOP");
    let gop = unsafe{ &mut *gop.get() };

    let graphics_mode = gop.current_mode_info();
    let (width, height) = graphics_mode.resolution();

    info!("[+] Current graphics resolution: {}x{}", width, height);

    let fb_addr = gop.frame_buffer().as_mut_ptr() as *mut u32;
    let fb_size = gop.frame_buffer().size();

    info!("[+] Frame buffer address: {:p}", fb_addr);
    info!("[+] Frame buffer size: {}", fb_size);

    for i in 0..width * height {
        unsafe {
            *fb_addr.offset(i as isize) = 0x000F0F0F;
        }
    }

    loop {}

}
