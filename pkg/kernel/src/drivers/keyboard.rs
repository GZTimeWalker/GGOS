#![allow(dead_code)]
use crate::drivers::{serial, console};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use alloc::string::String;
pub type DefaultKeyBoard = Keyboard<layouts::Us104Key, ScancodeSet1>;

const DEFAULT_BUF_SIZE: usize = 128;

once_mutex!(pub KEY_BUF: ArrayQueue<DecodedKey>);
once_mutex!(pub KEYBOARD: DefaultKeyBoard);

guard_access_fn!(pub get_key_buf(KEY_BUF: ArrayQueue<DecodedKey>));
guard_access_fn!(pub get_keyboard(KEYBOARD: DefaultKeyBoard));

pub fn init() {
    init_KEY_BUF(ArrayQueue::new(DEFAULT_BUF_SIZE));
    init_KEYBOARD(
        Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::Ignore
        )
    );
}

pub fn try_get_key() -> Option<DecodedKey> {
    interrupts::without_interrupts(|| {
        if let Some(key) = get_key_buf_for_sure().pop() {
            return Some(key);
        }
        None
    })
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
    while let DecodedKey::Unicode(k) = get_key() {
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
    println!();
    s
}
