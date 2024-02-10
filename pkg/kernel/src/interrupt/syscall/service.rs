use core::alloc::Layout;

use embedded_graphics::geometry::Point;

use crate::display::get_display_for_sure;
use crate::proc::*;
use crate::utils::*;

use super::SyscallArgs;

pub fn sys_clock() -> i64 {
    clock::now().timestamp_nanos_opt().unwrap_or_default()
}

pub fn sys_draw(args: &SyscallArgs) {
    let _ = get_display_for_sure().draw_pixel_u32(
        Point::new(args.arg0 as i32, args.arg1 as i32),
        args.arg2 as u32,
    );
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn spawn_process(args: &SyscallArgs) -> usize {
    let path = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };

    let file = crate::filesystem::try_get_file(path, fs::Mode::ReadOnly);

    if file.is_err() {
        warn!("spawn_process: file not found: {}", path);
        return 0;
    }

    let file = file.unwrap();

    let pid = spawn(&file);

    if pid.is_err() {
        warn!("spawn_process: failed to spawn process: {}", path);
        return 0;
    }

    pid.unwrap().0 as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    let fd = handle(args.arg0 as u8);
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
    let fd = handle(args.arg0 as u8);
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

pub fn sys_get_pid() -> u16 {
    current_pid().0
}

pub fn sys_fork(context: &mut ProcessContext) {
    fork(context)
}

pub fn sys_open(args: &SyscallArgs) -> usize {
    let path = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };

    let fd = open(path, args.arg2 as u8);

    if fd.is_none() {
        warn!("sys_open: failed to open: {}", path);
        return 0;
    }

    let fd = fd.unwrap();

    trace!("sys_open: opened: {} at fd={}", path, &fd);

    fd as usize
}

pub fn sys_close(args: &SyscallArgs) -> usize {
    close(args.arg0 as u8) as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    process_exit(args.arg0 as isize, context);
}

pub fn list_process() {
    print_process_list();
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

pub fn sys_wait_pid(args: &SyscallArgs) -> usize {
    let pid = ProcessId(args.arg0 as u16);
    let ret = wait_pid(pid);
    ret as usize
}

pub fn sys_kill(args: &SyscallArgs, context: &mut ProcessContext) {
    let pid = ProcessId(args.arg0 as u16);
    if pid == ProcessId(1) {
        warn!("sys_kill: cannot kill kernel!");
        return;
    }
    kill(pid, context);
}

pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2) as usize),
        1 => context.set_rax(remove_sem(args.arg1 as u32) as usize),
        2 => sem_up(args.arg1 as u32, context),
        3 => sem_down(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}
