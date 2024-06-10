use crate::*;
use alloc::string::*;
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

pub struct Random(u8);

impl Stdin {
    fn new() -> Self {
        Self
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

    pub fn read_char_with_buf(&self, buf: &mut [u8]) -> Option<char> {
        if let Some(bytes) = sys_read(0, buf) {
            if bytes > 0 {
                return Some(String::from_utf8_lossy(&buf[..bytes]).to_string().remove(0));
            }
        }
        None
    }

    pub fn read_line(&self) -> String {
        let mut string = String::new();
        let mut buf = [0; 4];
        loop {
            if let Some(k) = self.read_char_with_buf(&mut buf[..4]) {
                match k {
                    '\n' => {
                        stdout().write("\n");
                        break;
                    }
                    '\x03' => {
                        string.clear();
                        break;
                    }
                    '\x04' => {
                        string.clear();
                        string.push('\x04');
                        break;
                    }
                    '\x08' => {
                        if !string.is_empty() {
                            stdout().write("\x08");
                            string.pop();
                        }
                    }
                    // ignore other control characters
                    '\x00'..='\x1F' => {}
                    c => {
                        self::print!("{}", k);
                        string.push(c);
                    }
                }
            }
        }
        string
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

impl Random {
    pub fn new() -> Self {
        Self(sys_open("/dev/random", FileMode::ReadOnly))
    }

    pub fn next_u32(&self) -> u32 {
        let mut buf = vec![0; 4];
        if let Some(bytes) = sys_read(self.0, &mut buf) {
            if bytes > 0 {
                return u32::from_le_bytes(buf[..bytes].try_into().unwrap());
            }
        }
        0
    }

    pub fn next_u64(&self) -> u64 {
        let mut buf = vec![0; 8];
        if let Some(bytes) = sys_read(self.0, &mut buf) {
            if bytes > 0 {
                return u64::from_le_bytes(buf[..bytes].try_into().unwrap());
            }
        }
        0
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Random {
    fn drop(&mut self) {
        sys_close(self.0);
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

/// The different ways we can open a file.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum FileMode {
    /// Open a file for reading, if it exists.
    ReadOnly = 0,
    /// Open a file for appending (writing to the end of the existing file), if it exists.
    ReadWriteAppend = 1,
    /// Open a file and remove all contents, before writing to the start of the existing file, if it exists.
    ReadWriteTruncate = 2,
    /// Create a new empty file. Fail if it exists.
    ReadWriteCreate = 3,
    /// Create a new empty file, or truncate an existing file.
    ReadWriteCreateOrTruncate = 4,
    /// Create a new empty file, or append to an existing file.
    ReadWriteCreateOrAppend = 5,
}
