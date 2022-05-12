use core::alloc::Layout;

use crate::utils::Registers;
use crate::{display::get_display_for_sure, utils::*};
use embedded_graphics::prelude::*;
use x86_64::structures::idt::InterruptStackFrame;

use super::SyscallArgs;

pub fn sys_clock() -> i64 {
    clock::now().timestamp_nanos()
}

pub fn sys_draw(args: &SyscallArgs) {
    let _ = get_display_for_sure().draw_pixel_u32(
        Point::new(args.arg0 as i32, args.arg1 as i32),
        args.arg2 as u32,
    );
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };
    trace!("sys_allocate: \n{:#?}", layout);
    let ptr = crate::allocator::ALLOCATOR
        .lock()
        .allocate_first_fit(layout.clone())
        .unwrap()
        .as_ptr();
    trace!("allocated {:x}", ptr as u64);
    ptr as usize
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let ptr = args.arg0 as *mut u8;
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    unsafe {
        crate::allocator::ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), layout.clone());
    }
    trace!("deallocated {:x}", ptr as u64);
}

pub fn spawn_process(args: &SyscallArgs) -> usize {
    let path = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };

    let file = crate::filesystem::try_get_file(path);

    if file.is_err() {
        warn!("spawn_process: file not found: {}", path);
        return 0;
    }
    let file = file.unwrap();

    let pid = crate::process::spawn(&file);

    if pid.is_err() {
        warn!("spawn_process: failed to spawn process: {}", path);
        return 0;
    }
    u16::from(pid.unwrap()) as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    let fd = get_handle(args.arg0 as u8);
    if let Some(res) = fd {
        let buf = unsafe { core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2) };
        if let Ok(size) = res.read(buf) {
            size
        } else {
            0
        }
    } else {
        0
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    let fd = get_handle(args.arg0 as u8);
    if let Some(res) = fd {
        let buf = unsafe { core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2) };
        if let Ok(size) = res.write(buf) {
            size
        } else {
            0
        }
    } else {
        0
    }
}

pub fn exit_process(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    crate::process::process_exit(regs, sf);
}

pub fn list_process() {
    crate::process::print_process_list();
}

pub fn list_dir(args: &SyscallArgs) {
    let root = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };
    crate::filesystem::ls(root);
}

pub fn get_handle(fd: u8) -> Option<Resource> {
    crate::process::handle(fd)
}
