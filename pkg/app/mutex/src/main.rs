#![no_std]
#![no_main]

use lib::*;

extern crate lib;

mod sync;
use sync::SpinLock;

static mut LOCK: SpinLock = SpinLock::new();
static mut BURGER: isize = 0;

fn main() -> usize {
    let pid = sys_fork();

    if pid == 0 {
        unsafe {
            boy();
        }
    } else {
        unsafe {
            mother();
        }
    }

    0
}

unsafe fn mother() {
    LOCK.lock();

    println!("Mother: Start to make cheese burger, there are {} cheese burger now", &BURGER);

    BURGER += 10;

    println!("Mother: Oh, I have to hang clothes out.");

    let now = sys_time();
    let mut current = now;
    while current - now < lib::Duration::seconds(1) {
        current = sys_time();
    }

    println!("Mother: Oh, Jesus! There are {} cheese burgers", &BURGER);

    LOCK.unlock();
}

unsafe fn boy() {
    LOCK.lock();

    println!("Boy   : Look what I found!");
    BURGER -= 10;

    LOCK.unlock();
}

entry!(main);
