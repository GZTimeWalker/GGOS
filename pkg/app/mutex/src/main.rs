#![no_std]
#![no_main]

use lib::*;

extern crate lib;

static MUTEX: Semaphore = Semaphore::new(0x6666);
static LOCK: SpinLock = SpinLock::new();
static mut BURGER: isize = 0;
static mut BURGER_SEM: isize = 0;

fn main() -> isize {
    let pid = sys_fork();

    if pid == 0 {
        try_semaphore();
    } else {
        try_spin();
        sys_wait_pid(pid);
    }

    0
}

fn try_spin() {
    let pid = sys_fork();

    if pid == 0 {
        unsafe { boy_spin() };
    } else {
        unsafe { mother_spin() };
        sys_wait_pid(pid);
    }
}

unsafe fn mother_spin() {
    LOCK.acquire();
    let burger_ptr = &raw mut BURGER;

    println!(
        "Mother - SPIN : Start to make cheese burger, there are {} cheese burger now",
        *burger_ptr
    );

    *burger_ptr += 10;

    println!("Mother - SPIN : Oh, I have to hang clothes out.");

    sleep(1500);

    println!(
        "Mother - SPIN : Oh, Jesus! There are {} cheese burgers",
        *burger_ptr
    );

    LOCK.release();
}

unsafe fn boy_spin() {
    sleep(200);
    let burger_ptr = &raw mut BURGER;

    LOCK.acquire();

    println!("Boy    - SPIN : Look what I found!");
    *burger_ptr -= 10;

    LOCK.release();
}

fn try_semaphore() {
    MUTEX.init(1);

    let pid = sys_fork();

    if pid == 0 {
        unsafe { boy_semaphore() };
    } else {
        unsafe { mother_semaphore() };
        sys_wait_pid(pid);
        MUTEX.free();
    }
}

unsafe fn mother_semaphore() {
    MUTEX.wait();
    let burger_ptr = &raw mut BURGER_SEM;

    println!(
        "Mother - SEMA : Start to make cheese burger, there are {} cheese burger now",
        *burger_ptr
    );

    *burger_ptr += 10;

    println!("Mother - SEMA : Oh, I have to hang clothes out.");

    sleep(1500);

    println!(
        "Mother - SEMA : Oh, Jesus! There are {} cheese burgers",
        *burger_ptr
    );

    MUTEX.signal();
}

unsafe fn boy_semaphore() {
    sleep(200);
    let burger_ptr = &raw mut BURGER_SEM;

    MUTEX.wait();

    println!("Boy    - SEMA : Look what I found!");
    *burger_ptr -= 10;

    MUTEX.signal();
}

entry!(main);
