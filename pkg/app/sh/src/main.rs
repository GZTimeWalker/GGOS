#![no_std]
#![no_main]

extern crate alloc;

use lib::*;
use alloc::vec::Vec;
use alloc::string::*;
use lib::io::stdin;

extern crate lib;

fn main() {

    let mut root_dir = String::from("/APP/");

    loop {
        print!("[{}] ", root_dir);
        let input = stdin().read_line();
        let line: Vec<&str> = input.trim().split(' ').collect();
        match line[0] {
            "exit" => break,
            "ps" => sys_stat(),
            "ls" => sys_list_dir(root_dir.as_str()),
            "cat" => {
                // ggos::filesystem::cat(root_dir.as_str(), line[1]);
            }
            "cd" => {
                if line[1].starts_with("/") {
                    root_dir = String::from(line[1]);
                }
                match line[1] {
                    ".." => {
                        if root_dir.as_str() == "/" {
                            break;
                        }
                        root_dir.pop();
                        let pos = root_dir.rfind('/').unwrap();
                        root_dir = root_dir[..pos + 1].to_string();
                    },
                    "." => break,
                    _ => {
                        root_dir.push_str(line[1]);
                        root_dir.push('/');
                        root_dir = root_dir.to_ascii_uppercase();
                    }
                }
            }
            "exec" => {
                let path = root_dir.clone() + line[1];
                let start = sys_time();

                let pid = sys_spawn(path.as_str());
                if pid == 0 {
                    println!("[!] failed to spawn process: {}#{}", line[1], pid);
                    continue;
                } else {
                    println!("[+] spawned process: {}#{}", line[1], pid);
                }

                let ret = sys_wait_pid(pid);
                let time = sys_time() - start ;

                println!("[+] process exited with code {} @ {}s", ret, time.num_seconds());
            }
            _ => println!("[=] you said \"{}\"", input),
        }
    }

    lib::sys_exit(0);
}

entry!(main);
