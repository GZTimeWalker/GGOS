#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> usize {
    println!("Hello, world!!!");

    let time = lib::sys_time();
    println!("Now at: {} UTC", time);

    huge_stack();

    println!("Exiting...");

    233
}

#[inline(never)]
fn huge_stack() {
    println!("Huge stack testing...");

    let mut stack = [0u64; 0x1000];

    for i in 0..stack.len() {
        stack[i] = i as u64;
    }

    for i in 0..stack.len() / 256 {
        println!("{:#05x} == {:#05x}", i * 256, stack[i * 256]);
    }
}

entry!(main);
