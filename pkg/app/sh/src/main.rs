#![no_std]
#![no_main]

extern crate alloc;

mod consts;
mod services;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use lib::*;

extern crate lib;

fn main() -> usize {
    let mut root_dir = String::from("/APP/");
    println!("            <<< Welcome to GGOS shell >>>            ");
    println!("                                 type `help` for help");
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
            "nohup" => {
                if line.len() < 2 {
                    println!("Usage: nohup <file>");
                    continue;
                }

                services::nohup(line[1], root_dir.as_str());
            }
            "kill" => {
                if line.len() < 2 {
                    println!("Usage: kill <pid>");
                    continue;
                }
                let pid = line[1].to_string().parse::<u16>();

                if pid.is_err() {
                    errln!("Cannot parse pid");
                    continue;
                }

                services::kill(pid.unwrap());
            }
            "help" => print!("{}", consts::help_text()),
            _ => println!("[=] you said \"{}\"", input),
        }
    }

    0
}

entry!(main);
