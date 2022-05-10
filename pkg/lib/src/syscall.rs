use crate::Syscall;

pub fn sys_draw(x: i32, y: i32, color: u32) -> isize {
    syscall!(Syscall::Draw, x as usize, y as usize, color as usize)
}

pub fn sys_write(fd: u64, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(Syscall::Write, fd, buf.as_ptr() as u64, buf.len() as u64);
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

pub fn sys_read(fd: u64, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(Syscall::Read, fd, buf.as_ptr() as u64, buf.len() as u64);
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(
        Syscall::Allocate,
        layout as *const _
    ) as *mut u8
}

pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> isize {
    syscall!(
        Syscall::Deallocate,
        ptr,
        layout as *const _
    )
}

pub fn sys_exit(code: usize) {
    syscall!(Syscall::ExitProcess, code);
}
