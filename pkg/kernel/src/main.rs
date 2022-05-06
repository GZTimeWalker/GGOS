#![no_std]
#![no_main]

use ggos_kernel as ggos;
use ggos_kernel::*;

boot::entry_point!(kernal_main);

pub fn kernal_main(boot_info: &'static boot::BootInfo) -> ! {
    ggos::init(boot_info);

    let mut test_num = 0;

    loop {
        print!("[>] ");
        let line = input::get_line();
        match line.trim() {
            "exit" => break,
            "ps" => {
                ggos::process::print_process_list();
            }
            "t" => {
                ggos::new_test_thread(format!("{}", test_num).as_str());
                test_num += 1;
            }
            "ls" => {
                let root = fs::root_dir();
                ggos::filesystem::fs()
                    .iterate_dir(&root,
                        |entry| println!("{}", entry)
                    ).unwrap();
            }
            _ => println!("[=] {}", line),
        }
    }

    ggos::shutdown(boot_info);
}
