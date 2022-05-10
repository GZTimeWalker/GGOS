#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

fn main() {
    println!("Hello, world!");
    lib::sys_exit(0);
}

entry!(main);
