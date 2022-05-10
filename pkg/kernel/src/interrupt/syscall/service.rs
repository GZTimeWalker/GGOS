use crate::{utils::*, display::get_display_for_sure};
use embedded_graphics::prelude::*;
use x86_64::structures::idt::InterruptStackFrame;
use crate::utils::Registers;

pub fn sys_clock() -> i64 {
    clock::now().timestamp_nanos()
}

pub fn sys_draw(x: usize, y: usize, color: usize) {
    let _ = get_display_for_sure().draw_pixel_u32(
        Point::new(x as i32, y as i32),
        color as u32
    );
}

pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    debug!("sys_allocate: \n{:#?}", layout);
    let ptr = crate::allocator::ALLOCATOR
        .lock()
        .allocate_first_fit(layout.clone())
        .unwrap()
        .as_ptr();
    debug!("allocated {:x}", ptr as u64);
    ptr
}

pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) {
    unsafe {
        crate::allocator::ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), layout.clone());
    }
    debug!("deallocated {:x}", ptr as u64);
}

pub fn exit_process(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    crate::process::process_exit(regs, sf);
}
