#![no_std]
#![no_main]

use alloc::string::*;
use alloc::vec::Vec;
use ggos_kernel as ggos;
use ggos::*;

extern crate alloc;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let mut test_num = 0;
    let mut root_dir = String::from("/");

    loop {
        print!("[{}] ", root_dir);
        let input = input::get_line();
        let line: Vec<&str> = input.trim().split(' ').collect();
        match line[0] {
            "exit" => break,
            "ps" => {
                ggos::process::print_process_list();
            }
            "t" => {
                ggos::new_test_thread(format!("{}", test_num).as_str());
                test_num += 1;
            }
            "ls" => {
                ggos::filesystem::ls(root_dir.as_str());
            }
            "cat" => {
                ggos::filesystem::cat(root_dir.as_str(), line[1]);
            }
            "cd" => {
                match line[1] {
                    ".." => {
                        if root_dir.as_str() == "/" {
                            break;
                        }
                        root_dir.pop();
                        let pos = root_dir.rfind('/').unwrap();
                        root_dir = root_dir[..pos + 1].to_string();
                    },
                    _ => {
                        root_dir.push_str(line[1]);
                        root_dir.push('/');
                        root_dir = root_dir.to_ascii_uppercase();
                    }
                }
            }
            _ => println!("[=] {}", input),
        }
    }

    ggos::shutdown(boot_info);
}
