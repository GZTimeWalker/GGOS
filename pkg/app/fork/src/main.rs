#![no_std]
#![no_main]

extern crate alloc;
extern crate lib;

use lib::*;

static mut M: u64 = 0xdeadbeef;

fn main() -> isize {
    let mut c = 32;
    let m_ptr = &raw mut M;

    // do not alloc heap before `fork`
    // which may cause unexpected behavior since we won't copy the heap in `fork`
    let pid = sys_fork();

    if pid == 0 {
        println!("I am the child process");

        assert_eq!(c, 32);

        unsafe {
            println!("child read value of M: {:#x}", *m_ptr);
            *m_ptr = 0x2333;
            println!("child changed the value of M: {:#x}", *m_ptr);
        }

        c += 32;
    } else {
        println!("I am the parent process");

        sys_stat();

        assert_eq!(c, 32);

        println!("Waiting for child to exit...");

        let ret = sys_wait_pid(pid);

        println!("Child exited with status {}", ret);

        assert_eq!(ret, 64);

        unsafe {
            println!("parent read value of M: {:#x}", *m_ptr);
            assert_eq!(*m_ptr, 0x2333);
        }

        c += 1024;

        assert_eq!(c, 1056);
    }

    c
}

entry!(main);
