#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() {
    println!("Hello, world!!!");
    let time = lib::sys_time();
    println!("Now at: {}", time);
    lib::sys_exit(0);
}

entry!(main);
