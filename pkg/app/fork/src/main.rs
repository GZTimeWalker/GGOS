#![no_std]
#![no_main]

extern crate alloc;
extern crate lib;

use lib::*;

fn main() {
    let mut c = 23;
    let ret = sys_fork();

    if ret == 0 {
        // println!("I am the child process");
        // println!("Exiting...");
        c += 32;
    } else {
        // println!("I am the parent process");
        // println!("Waiting for child to exit...");
        // let ret = sys_wait_pid(ret);
        // println!("Child exited with status {}", ret);
        c += 24;
    }
    unsafe {
        core::arch::asm!("hlt");
    }
    sys_exit(c);
}

entry!(main);
