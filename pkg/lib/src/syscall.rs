use crate::Syscall;
use chrono::naive::*;

pub fn sys_draw(x: i32, y: i32, color: u32) -> usize {
    syscall!(Syscall::Draw, x as usize, y as usize, color as usize)
}

pub fn sys_write(fd: u8, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(Syscall::Write, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

pub fn sys_read(fd: u8, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(Syscall::Read, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(Syscall::Allocate, layout as *const _) as *mut u8
}

pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> usize {
    syscall!(Syscall::Deallocate, ptr, layout as *const _)
}

pub fn sys_exit(code: usize) {
    syscall!(Syscall::ExitProcess, code);
}

pub fn sys_wait_pid(pid: u16) -> isize {
    loop {
        let ret = syscall!(Syscall::WaitPid, pid as u64) as isize;
        if !ret.is_negative() {
            return ret;
        }
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

pub fn sys_time() -> NaiveDateTime {
    let time = syscall!(Syscall::Time) as i64;
    const BILLION: i64 = 1_000_000_000;
    NaiveDateTime::from_timestamp(time / BILLION, (time % BILLION) as u32)
}

pub fn sys_list_dir(root: &str) {
    syscall!(
        Syscall::DirectoryList,
        root.as_ptr() as u64,
        root.len() as u64
    );
}

pub fn sys_stat() {
    syscall!(Syscall::Stat);
}

pub fn sys_spawn(path: &str) -> u16 {
    let pid = syscall!(
        Syscall::SpawnProcess,
        path.as_ptr() as u64,
        path.len() as u64
    ) as u16;
    unsafe {
        core::arch::asm!("hlt");
    }
    pid
}

pub fn sys_open(path: &str, mode: crate::FileMode) -> u8 {
    syscall!(
        Syscall::Open,
        path.as_ptr() as u64,
        path.len() as u64,
        mode as u64
    ) as u8
}

pub fn sys_close(fd: u8) -> bool {
    syscall!(Syscall::Close, fd as u64) != 0
}
