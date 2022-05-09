use kernel::Syscall;

fn syscall(id: Syscall, arg0: u64, arg1: u64, arg2: u64) {
    unsafe {
        core::arch::asm!("int 0x80", in("rax") id as u64, in("rbx") arg0, in("rcx") arg1, in("rdx") arg2);
    }
}

pub fn sys_draw(x: i32, y: i32, color: u32) {
    syscall(Syscall::Draw, x as u64, y as u64, color as u64);
}

pub fn sys_write(s: &str, fd: usize) {
    syscall(Syscall::Write, s.as_ptr() as u64, s.len() as u64, fd)
}

pub fn sys_read(buf: &mut [u8], count: usize, fd: usize) -> usize {
    syscall(Syscall::Read, buf.as_ptr() as u64, count as u64, fd)
}
