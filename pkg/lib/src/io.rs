use crate::*;
use alloc::vec;
use alloc::string::{String, ToString};

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self {}
    }

    pub fn read_char(&self) -> Option<char> {
        let mut buf = vec![0; 4];
        if let Some(bytes) = sys_read(0, &mut buf) {
            if bytes > 0 {
                return Some(String::from_utf8_lossy(&buf[..bytes]).to_string().remove(0));
            }
        }
        None
    }

    pub fn read_line(&self) -> String {
        let mut string = String::new();
        loop {
            if let Some(bytes) = self.read_char() {
                if bytes == '\n' {
                    break;
                } else {
                    string.push(bytes);
                }
            }
        }
        string
    }
}

impl Stdout {
    fn new() -> Self {
        Self {}
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self {}
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
