#![no_std]
#![no_main]

extern crate alloc;
extern crate lib;
use lib::*;

static mut M: u64 = 0xdeadbeef;

fn main() -> usize {
    let mut c = 32;

    // do not alloc heap before `fork`
    // which may cause unexpected behavior since we won't copy the heap in `fork`
    let ret = sys_fork();

    if ret == 0 {
        println!("I am the child process");
        unsafe {
            println!("child read value of M: {:#x}", M);
            M = 0x2333;
            println!("child changed the value of M: {:#x}", M);
        }
        c += 32;
    } else {
        println!("I am the parent process");

        sys_stat();

        println!("Waiting for child to exit...");

        let ret = sys_wait_pid(ret);

        println!("Child exited with status {}", ret);

        unsafe {
            println!("parent read value of M: {:#x}", M);
        }

        c += 1024;
    }

    c
}

entry!(main);
