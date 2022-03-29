#![allow(dead_code)]

use crate::utils::colors;
use crate::display::get_display_for_sure;
use crate::utils::JBMONO;
use core::fmt::*;
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoFont},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Text, renderer::CharacterStyle},
    primitives::{Line, PrimitiveStyle}
};

once_mutex!(pub CONSOLE: Console);

const FONT: &MonoFont = &JBMONO;

const FONT_X: u8 = FONT.character_size.width as u8;
const FONT_Y: u8 = FONT.character_size.height as u8;
const SPACING: u8 = FONT.character_spacing as u8;

const TOP_PAD_LINE_NUM: usize = 3;

pub fn initialize() {
    init_CONSOLE(Console::new());
    get_console_for_sure().clear();
}

guard_access_fn!(pub get_console(CONSOLE: Console));

pub struct Console {
    x_pos: usize,
    y_pos: usize,
    frontground: Rgb888,
    background: Rgb888
}

impl Console {
    pub fn new() -> Self {
        Self {
            x_pos: 0,
            y_pos: 0,
            frontground: colors::FRONTGROUND,
            background: colors::BACKGROUND
        }
    }
}

impl Console {
    pub fn size(&self) -> (usize, usize) {
        let size: Size = get_display_for_sure().size();
        (
            size.width as usize / (FONT_X + SPACING) as usize,
            size.height as usize / FONT_Y as usize - TOP_PAD_LINE_NUM
        )
    }

    fn get_char_pos(&self, x: usize, y: usize) -> (usize, usize) {
        (x * FONT_X as usize, (y + TOP_PAD_LINE_NUM) * FONT_Y as usize)
    }

    pub fn next_row(&mut self) {
        self.y_pos += 1;
        if self.y_pos > self.size().1 {
            self.scroll();
            self.y_pos = self.size().1;
        }
        self.x_pos = 0;
    }

    pub fn next_char(&mut self) {
        self.x_pos += 1;
        if self.x_pos > self.size().0 {
            self.next_row()
        }
    }

    pub fn scroll(&self) {
        get_display_for_sure().scrollup(
            Some(self.background),
            FONT_Y,
            FONT_Y as usize * (TOP_PAD_LINE_NUM - 1)
        );
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char) {
        let mut buf = [0u8; 2];
        let str_c = c.encode_utf8(&mut buf);
        let pos = Point::new(
            x as i32 * (FONT_X + SPACING) as i32,
            (y + TOP_PAD_LINE_NUM) as i32 * FONT_Y as i32
        );
        let mut style = MonoTextStyle::new(FONT, self.frontground);
        CharacterStyle::set_background_color(&mut style, Some(self.background));
        Text::new( str_c, pos, style)
        .draw(&mut *get_display_for_sure())
        .expect("Writing Error!");
    }

    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.next_row();
                },
                '\r' => self.x_pos = 0,
                '\x08' => self.x_pos -= 1,
                _ => {
                    self.write_char_at(self.x_pos, self.y_pos, c);
                    self.next_char()
                }
            }
        }
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        self.x_pos = (self.x_pos as isize + dx) as usize;
        self.y_pos = (self.y_pos as isize + dy) as usize;
    }

    pub fn draw_hint(&mut self) {
        let (x, y) = (self.x_pos, self.y_pos);
        let (cx, cy) = self.get_char_pos(x, y);
        Line::new(
            Point::new(cx as i32, cy as i32),
            Point::new(cx as i32, cy as i32 + FONT_Y as i32 - 1),
        ).into_styled(
            PrimitiveStyle::with_stroke(colors::FRONTGROUND, 2)
        ).draw(&mut *get_display_for_sure())
        .expect("Hint Drawing Error!");
    }

    pub fn set_color(&mut self, front: Option<Rgb888>, back: Option<Rgb888>) {
        if let Some(color) = front {
            self.frontground = color;
        }
        if let Some(color) = back {
            self.background = color;
        }
    }

    pub fn clear(&self) {
        get_display_for_sure().clear(
            Some(self.background),
            FONT_Y as usize * TOP_PAD_LINE_NUM
        );
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::console::print_internal(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! print_warn {
    ($($arg:tt)*) => ($crate::console::print_warn_internal(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_warn {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print_warn!("{}\n", format_args!($($arg)*)));
}


#[doc(hidden)]
pub fn print_internal(args: Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        get_console_for_sure().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn print_warn_internal(args: Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        let mut console = get_console_for_sure();
        console.set_color(Some(colors::RED), None);
        console.write_fmt(args).unwrap();
        console.set_color(Some(colors::FRONTGROUND), None);
    });
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print_warn!("[!] {}", info);
    loop {}
}
