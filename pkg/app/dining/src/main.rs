#![no_std]
#![no_main]

use lib::*;

extern crate lib;

static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];

fn main() -> usize {
    let mut pids = [0u16; 5];

    for i in 0..5 {
        CHOPSTICK[i].init(1);
    }

    for i in 0..5 {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(i);
        } else {
            pids[i] = pid;
        }
    }

    let cpid = sys_get_pid();

    println!("#{} holds threads: {:?}", cpid, &pids);

    sys_stat();

    for i in 0..5 {
        println!("#{} Waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    0
}

fn philosopher(id: usize) -> ! {
    let pid = sys_get_pid();
    for _ in 0..20 {
        if id == 0 {
            println!("philosopher #{}({}) is sleeping...", id, pid);
            core::hint::spin_loop();
        }

        println!("philosopher #{}({}) is thinking...", id, pid);

        CHOPSTICK[id].acquire();
        CHOPSTICK[(id + 1) % 5].acquire();

        println!("philosopher #{}({}) is eating...", id, pid);

        CHOPSTICK[(id + 1) % 5].release();
        CHOPSTICK[id].release();
    }
    sys_exit(0);
}

entry!(main);
