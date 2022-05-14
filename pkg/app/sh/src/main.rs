#![no_std]
#![no_main]

extern crate alloc;

mod cat;

use lib::*;
use alloc::vec;
use alloc::vec::*;
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
                if line.len() < 2 {
                    println!("Usage: cat <file> | /dev/random");
                    continue;
                }

                let path = if line[1].starts_with('/') {
                    String::from(line[1])
                } else {
                    root_dir.clone() + line[1]
                };

                let fd = sys_open(path.as_str(), FileMode::ReadOnly);

                if fd == 0 {
                    errln!("File not found or cannot open");
                    continue;
                }

                let mut buf = if path == "/dev/random" {
                    vec![0; 24]
                } else {
                    vec![0; 0x2000]
                };

                let size = sys_read(fd, &mut buf);

                if size.is_none() {
                    errln!("Cannot read file");
                    continue;
                }

                let size = size.unwrap();
                if size == 0 {
                    errln!("File is empty or buffer is too small!");
                    continue;
                }

                cat::cat(&buf[..size]);

                sys_close(fd);
            }
            "cd" => {
                if line.len() < 2 {
                    println!("Usage: cd <dir>");
                    continue;
                }

                if line[1].starts_with("/") {
                    root_dir = String::from(line[1]);
                    continue;
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
                if line.len() < 2 {
                    println!("Usage: exec <file>");
                    continue;
                }

                let path = root_dir.clone() + line[1];
                let start = sys_time();

                let pid = sys_spawn(path.as_str());
                if pid == 0 {
                    errln!("[!] failed to spawn process: {}", line[1]);
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
