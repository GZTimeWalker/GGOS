use alloc::{format, string::*, vec};
use lib::*;

pub fn show_hex(data: &[u8]) {
    let mut string = String::with_capacity(data.len() * 3);

    let mut count = 0;
    for (idx, b) in data.iter().enumerate() {
        if count == 0 {
            string.push_str("    ");
        }
        string.push_str(&format!("{:02x}", b));
        count += 1;
        if count % 8 == 0 {
            string.push(' ');
        }
        if count == 24 {
            string.push_str(" | ");
            for d in data.iter().take(idx + 1).skip(idx - 23) {
                if (*d as char).is_ascii_graphic() || *d == 0x20 {
                    string.push(*d as char);
                } else {
                    string.push('.');
                }
            }
            string.push('\n');
            count = 0;
        }
    }
    if count > 0 {
        for _ in count..24 {
            string.push_str("  ");
        }
        for _ in 0..3 - (count / 8) {
            string.push(' ');
        }
        string.push_str(" | ");
        for d in data.iter().skip(data.len() - count) {
            if (*d as char).is_ascii_graphic() || *d == 0x20 {
                string.push(*d as char);
            } else {
                string.push('.');
            }
        }
        string.push('\n');
    }

    stdout().write(&string);
}

pub fn cat(path: &str, root_dir: &str) {
    let path = if path.starts_with('/') {
        String::from(path)
    } else {
        format!("{}{}", root_dir, path)
    }
    .to_ascii_uppercase();

    let fd = sys_open(path.as_str(), FileMode::ReadOnly);

    if fd == 0 {
        errln!("File not found or cannot open");
        return;
    }

    let mut buf = if path == "/dev/random" {
        vec![0; 24]
    } else {
        vec![0; 3072]
    };

    let mut bytes_read = 0;

    loop {
        if let Some(size) = sys_read(fd, &mut buf) {
            show_hex(&buf[..size]);
            bytes_read += size;
            if size < buf.len() {
                break;
            }
        } else {
            errln!("Cannot read file");
            return;
        }
    }

    sys_close(fd);

    println!("    > Read {} bytes from {}.", bytes_read, path);
}

pub fn cd(path: &str, root_dir: &mut String) {
    if path.starts_with('/') {
        *root_dir = String::from(path).to_ascii_uppercase();
        if !root_dir.ends_with('/') {
            root_dir.push('/');
        }
    } else {
        root_dir.push_str(path);
        root_dir.push('/');
        *root_dir = root_dir.to_ascii_uppercase();
    }
    canonicalize(root_dir)
}

pub fn exec(path: &str, root_dir: &str) {
    let path = format!("{}{}", root_dir, path).to_ascii_uppercase();
    let start = sys_time();

    let pid = sys_spawn(path.as_str());

    if pid == 0 {
        errln!("failed to spawn process: {}", path);
        return;
    }

    let ret = sys_wait_pid(pid);
    let time = sys_time() - start;

    println!(
        "[+] process exited with code {} @ {}s",
        ret,
        time.num_seconds()
    );
}

pub fn nohup(path: &str, root_dir: &str) {
    let path = format!("{}{}", root_dir, path).to_ascii_uppercase();

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

pub fn canonicalize(path: &mut String) {
    // If the path is not absolute, return an error
    if !path.starts_with('/') {
        *path = String::from("/");
        return;
    }

    // Create an empty string to store the canonicalized path
    let mut canonical = String::from("/");

    // Split the path by the separator and iterate over the segments
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if canonical.len() > 1 {
                    canonical.pop();
                    let last_index = canonical.rfind('/').unwrap_or(0);
                    canonical.truncate(last_index + 1);
                }
            }
            _ => {
                if canonical.len() > 1 {
                    canonical.push('/');
                }
                canonical.push_str(segment);
            }
        }
    }

    if canonical.len() > 1 {
        canonical.push('/');
    }

    *path = canonical;
}
