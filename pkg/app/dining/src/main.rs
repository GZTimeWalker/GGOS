#![no_std]
#![no_main]

use lib::*;

extern crate lib;

static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
static WAITER: Semaphore = Semaphore::new(64);

fn main() -> isize {
    let mut pids = [0u16; 5];

    // allow 4 philosophers to eat at the same time
    WAITER.init(4);

    for chop in &CHOPSTICK {
        chop.init(1);
    }

    for (i, item) in pids.iter_mut().enumerate() {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(i);
        } else {
            *item = pid;
        }
    }

    let cpid = sys_get_pid();

    println!("#{} holds threads: {:?}", cpid, &pids);

    sys_stat();

    for pid in pids {
        println!("#{} Waiting for #{}...", cpid, pid);
        sys_wait_pid(pid);
    }

    0
}

fn philosopher(id: usize) -> ! {
    let pid = sys_get_pid();

    for _ in 0..100 {
        // thinking
        println!("philosopher #{} ({}) is thinking...", id, pid);
        delay();

        // hungry
        WAITER.wait();
        CHOPSTICK[id].wait();
        CHOPSTICK[(id + 1) % 5].wait();
        println!("philosopher #{} ({}) is eating...", id, pid);
        CHOPSTICK[(id + 1) % 5].signal();
        CHOPSTICK[id].signal();
        WAITER.signal();
    }
    sys_exit(0);
}

#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..100 {
        core::hint::spin_loop();
    }
}

entry!(main);
