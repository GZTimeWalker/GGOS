#![no_std]
#![no_main]

extern crate alloc;

mod services;
mod consts;

use lib::*;
use alloc::vec::Vec;
use alloc::string::String;

extern crate lib;

fn main() {
    let mut root_dir = String::from("/APP/");
    println!("<<< Welcome to GGOS shell >>>");
    loop {
        print!("[{}] ", root_dir);
        let input = stdin().read_line();
        let line: Vec<&str> = input.trim().split(' ').collect();
        match line[0] {
            "exit" => break,
            "ps" => sys_stat(),
            "ls" => sys_list_dir(root_dir.as_str()),
            "cat" => {
                if line.len() < 2 {
                    println!("Usage: cat <file>");
                    continue;
                }

                services::cat(line[1], root_dir.as_str());
            }
            "cd" => {
                if line.len() < 2 {
                    println!("Usage: cd <dir>");
                    continue;
                }

                services::cd(line[1], &mut root_dir);
            }
            "exec" => {
                if line.len() < 2 {
                    println!("Usage: exec <file>");
                    continue;
                }

                services::exec(line[1], root_dir.as_str());
            }
            "help" => print!("{}", consts::help_text()),
            _ => println!("[=] you said \"{}\"", input),
        }
    }

    lib::sys_exit(0);
}

entry!(main);
