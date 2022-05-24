#![no_std]
#![no_main]

use lib::*;

extern crate lib;

static mut LOCK: SpinLock = SpinLock::new();
static mut BURGER: isize = 0;
static mut BURGER_SEM: isize = 0;

fn main() -> usize {
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
    LOCK.lock();

    println!(
        "Mother - SPIN : Start to make cheese burger, there are {} cheese burger now",
        &BURGER
    );

    BURGER += 10;

    println!("Mother - SPIN : Oh, I have to hang clothes out.");

    sleep(1500);

    println!(
        "Mother - SPIN : Oh, Jesus! There are {} cheese burgers",
        &BURGER
    );

    LOCK.unlock();
}

unsafe fn boy_spin() {
    sleep(200);

    LOCK.lock();

    println!("Boy    - SPIN : Look what I found!");
    BURGER -= 10;

    LOCK.unlock();
}

fn try_semaphore() {
    sys_new_sem(0x2323, 1);

    let pid = sys_fork();

    if pid == 0 {
        unsafe { boy_semaphore() };
    } else {
        unsafe { mother_semaphore() };
        sys_wait_pid(pid);
        sys_rm_sem(0x2323);
    }
}

unsafe fn mother_semaphore() {
    sys_sem_down(0x2323);

    println!(
        "Mother - SEMA : Start to make cheese burger, there are {} cheese burger now",
        &BURGER_SEM
    );

    BURGER_SEM += 10;

    println!("Mother - SEMA : Oh, I have to hang clothes out.");

    sleep(1500);

    println!(
        "Mother - SEMA : Oh, Jesus! There are {} cheese burgers",
        &BURGER_SEM
    );

    sys_sem_up(0x2323);
}

unsafe fn boy_semaphore() {
    sleep(200);

    sys_sem_down(0x2323);

    println!("Boy    - SEMA : Look what I found!");
    BURGER_SEM -= 10;

    sys_sem_up(0x2323);
}

entry!(main);
