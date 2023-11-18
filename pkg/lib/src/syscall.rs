use chrono::naive::*;
use syscall_def::Syscall;

#[inline(always)]
pub fn sys_draw(x: i32, y: i32, color: u32) -> usize {
    syscall!(Syscall::Draw, x as usize, y as usize, color as usize)
}

#[inline(always)]
pub fn sys_write(fd: u8, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Write,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_read(fd: u8, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Read,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(Syscall::Allocate, layout as *const _) as *mut u8
}

#[inline(always)]
pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> usize {
    syscall!(Syscall::Deallocate, ptr, layout as *const _)
}

#[inline(always)]
pub fn sys_exit(code: usize) -> ! {
    syscall!(Syscall::Exit, code);
    unreachable!();
}

#[inline(always)]
pub fn sys_wait_pid(pid: u16) -> isize {
    loop {
        let ret = syscall!(Syscall::WaitPid, pid as u64) as isize;
        if !ret.is_negative() {
            return ret;
        }
    }
}

#[inline(always)]
pub fn sys_time() -> NaiveDateTime {
    let time = syscall!(Syscall::Time) as i64;
    const BILLION: i64 = 1_000_000_000;
    NaiveDateTime::from_timestamp_opt(time / BILLION, (time % BILLION) as u32).unwrap_or_default()
}

#[inline(always)]
pub fn sys_list_dir(root: &str) {
    syscall!(Syscall::ListDir, root.as_ptr() as u64, root.len() as u64);
}

#[inline(always)]
pub fn sys_stat() {
    syscall!(Syscall::Stat);
}

#[inline(always)]
pub fn sys_spawn(path: &str) -> u16 {
    let pid = syscall!(Syscall::Spawn, path.as_ptr() as u64, path.len() as u64) as u16;
    pid
}

#[inline(always)]
pub fn sys_open(path: &str, mode: crate::FileMode) -> u8 {
    syscall!(
        Syscall::Open,
        path.as_ptr() as u64,
        path.len() as u64,
        mode as u64
    ) as u8
}

#[inline(always)]
pub fn sys_close(fd: u8) -> bool {
    syscall!(Syscall::Close, fd as u64) != 0
}

#[inline(always)]
pub fn sys_get_pid() -> u16 {
    syscall!(Syscall::GetPid) as u16
}

#[inline(always)]
pub fn sys_fork() -> u16 {
    let pid = syscall!(Syscall::VFork);
    pid as u16
}

#[inline(always)]
pub fn sys_kill(pid: u16) {
    syscall!(Syscall::Kill, pid as u64);
}

#[inline(always)]
pub fn sys_new_sem(key: u32, value: usize) -> isize {
    syscall!(Syscall::Sem, 0, key as usize, value) as isize
}

#[inline(always)]
pub fn sys_sem_up(key: u32) -> isize {
    syscall!(Syscall::Sem, 1, key as usize) as isize
}

#[inline(always)]
pub fn sys_sem_down(key: u32) -> isize {
    syscall!(Syscall::Sem, 2, key as usize) as isize
}

#[inline(always)]
pub fn sys_rm_sem(key: u32) -> isize {
    syscall!(Syscall::Sem, 3, key as usize) as isize
}
