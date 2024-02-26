use crate::drivers::{console, serial};
use alloc::string::String;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::DecodedKey;

const DEFAULT_BUF_SIZE: usize = 128;

type Key = DecodedKey;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(DEFAULT_BUF_SIZE);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_get_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn get_key() -> DecodedKey {
    loop {
        if let Some(k) = try_get_key() {
            return k;
        }
    }
}

pub fn get_line() -> String {
    let mut s = String::with_capacity(DEFAULT_BUF_SIZE);
    loop {
        let key = get_key();
        if let DecodedKey::Unicode(k) = key {
            match k {
                '\n' => break,
                '\x08' => {
                    if !s.is_empty() {
                        console::backspace();
                        serial::backspace();
                        s.pop(); // remove previous char
                    }
                }
                c => {
                    print!("{}", k);
                    s.push(c)
                }
            }
        }
        console::get_console_for_sure().draw_hint();
    }
    println!();
    s
}
