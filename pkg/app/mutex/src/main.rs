#![no_std]
#![no_main]

use lib::*;

extern crate lib;

mod sync;
use sync::SpinLock;

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

    let now = sys_time();
    let mut current = now;
    while current - now < lib::Duration::seconds(3) {
        core::hint::spin_loop();
        current = sys_time();
    }

    println!(
        "Mother - SPIN : Oh, Jesus! There are {} cheese burgers",
        &BURGER
    );

    LOCK.unlock();
}

unsafe fn boy_spin() {

    let now = sys_time();
    let mut current = now;
    while current - now < lib::Duration::milliseconds(200) {
        core::hint::spin_loop();
        current = sys_time();
    }

    LOCK.lock();

    println!("Boy    - SPIN : Look what I found!");
    BURGER -= 10;

    LOCK.unlock();
}

fn try_semaphore() {
    sys_new_sem(0x2323);

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

    let now = sys_time();
    let mut current = now;
    while current - now < lib::Duration::seconds(3) {
        core::hint::spin_loop();
        current = sys_time();
    }

    println!(
        "Mother - SEMA : Oh, Jesus! There are {} cheese burgers",
        &BURGER_SEM
    );

    sys_sem_up(0x2323);
}

unsafe fn boy_semaphore() {

    let now = sys_time();
    let mut current = now;
    while current - now < lib::Duration::milliseconds(200) {
        core::hint::spin_loop();
        current = sys_time();
    }

    sys_sem_down(0x2323);

    println!("Boy    - SEMA : Look what I found!");
    BURGER_SEM -= 10;

    sys_sem_up(0x2323);
}

entry!(main);
