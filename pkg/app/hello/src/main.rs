#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> usize {
    println!("Hello, world!!!");
    let time = lib::sys_time();
    println!("Now at: {}", time);
    println!("Exiting...");

    233
}

entry!(main);
