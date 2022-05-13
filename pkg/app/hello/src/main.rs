#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() {
    println!("Hello, world!!!");
    let time = lib::sys_time();
    println!("Now at: {}", time);
    println!("Exiting...");
    lib::sys_exit(233);
}

entry!(main);
