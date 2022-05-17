use alloc::{string::*, format, vec};
use lib::*;

pub fn show_hex(data: &[u8]) {
    let mut count = 0;
    for (idx, b) in data.iter().enumerate() {
        if count == 0 {
            print!("    ");
        }
        print!("{:02x}", b);
        count += 1;
        if count % 8 == 0 {
            print!(" ");
        }
        if count == 24 {
            print!(" | ");
            for i in idx - 23..=idx {
                let d = data[i];
                if (d as char).is_ascii_graphic() || d == 0x20 {
                    print!("{}", d as char);
                } else {
                    print!(".");
                }
            }
            println!();
            count = 0;
        }
    }
    if count > 0 {
        for _ in count..24 {
            print!("  ");
        }
        for _ in 0..3 - (count / 8) {
            print!(" ");
        }
        print!(" | ");
        for i in data.len() - count..data.len() {
            let d = data[i];
            if (d as char).is_ascii_graphic() || d == 0x20 {
                print!("{}", d as char);
            } else {
                print!(".");
            }
        }
        println!();
    }
}

pub fn cat(path: &str, root_dir: &str) {
    let path = if path.starts_with('/') {
        String::from(path)
    } else {
        format!("{}{}", root_dir, path)
    };

    let fd = sys_open(path.as_str(), FileMode::ReadOnly);

    if fd == 0 {
        errln!("File not found or cannot open");
        return;
    }

    let mut buf = if path == "/dev/random" {
        vec![0; 24]
    } else {
        vec![0; 0x4000]
    };

    let size = sys_read(fd, &mut buf);

    if size.is_none() {
        errln!("Cannot read file");
        return;
    }

    let size = size.unwrap();
    if size == 0 {
        errln!("File is empty or buffer is too small!");
        return;
    }

    show_hex(&buf[..size]);
    sys_close(fd);
}

pub fn cd(path: &str, root_dir: &mut String) {
    if path.starts_with("/") {
        *root_dir = String::from(path);
        return;
    }

    match path {
        ".." => {
            if root_dir.as_str() == "/" {
                return;
            }
            root_dir.pop();
            let pos = root_dir.rfind('/').unwrap();
            *root_dir = root_dir[..pos + 1].to_string();
        },
        "." => return,
        _ => {
            root_dir.push_str(path);
            root_dir.push('/');
            *root_dir = root_dir.to_ascii_uppercase();
        }
    }
}

pub fn exec(path: &str, root_dir: &str) {
    let path = format!("{}{}", root_dir, path);
    let start = sys_time();

    let pid = sys_spawn(path.as_str());

    if pid == 0 {
        errln!("failed to spawn process: {}", path);
        return;
    }

    let ret = sys_wait_pid(pid);
    let time = sys_time() - start ;

    println!("[+] process exited with code {} @ {}s", ret, time.num_seconds());
}

pub fn nohup(path: &str, root_dir: &str) {
    let path = format!("{}{}", root_dir, path);

    let pid = sys_spawn(path.as_str());

    if pid == 0 {
        errln!("failed to spawn process: {}", path);
        return;
    }

    println!("[+] process {}#{} spawned", path, pid);
}

pub fn kill(pid: u16) {
    sys_kill(pid);
}
