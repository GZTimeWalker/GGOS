use crate::drivers::display::get_display_for_sure;
use crate::utils::colors;
use crate::utils::font;
use alloc::vec::Vec;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::{renderer::CharacterStyle, Baseline, Text},
};
use fs::*;

once_mutex!(pub CONSOLE: Console);

const FONT: &MonoFont = &font::JBMONO;

const FONT_X: u8 = FONT.character_size.width as u8;
const FONT_Y: u8 = FONT.character_size.height as u8;
const SPACING: u8 = FONT.character_spacing as u8;

const TOP_PAD_LINE_NUM: isize = 3;

pub fn init() {
    init_CONSOLE(Console::new());
    let console = get_console_for_sure();
    console.clear();
    console.header();

    info!("Console Initialized.");
}

guard_access_fn!(pub get_console(CONSOLE: Console));

pub struct Console {
    x_pos: isize,
    y_pos: isize,
    frontground: Rgb888,
    background: Rgb888,
}

impl Console {
    pub fn new() -> Self {
        Self {
            x_pos: 0,
            y_pos: 0,
            frontground: colors::FRONTGROUND,
            background: colors::BACKGROUND,
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    pub fn size(&self) -> (isize, isize) {
        let size: Size = get_display_for_sure().size();
        (
            size.width as isize / (FONT_X + SPACING) as isize,
            size.height as isize / FONT_Y as isize - TOP_PAD_LINE_NUM,
        )
    }

    pub fn get_pos(&self) -> (isize, isize) {
        (self.x_pos, self.y_pos)
    }

    fn get_char_pos(&self, x: isize, y: isize) -> (isize, isize) {
        (
            x * FONT_X as isize,
            (y + TOP_PAD_LINE_NUM) * FONT_Y as isize,
        )
    }

    pub fn next_row(&mut self) {
        self.y_pos += 1;
        if self.y_pos > self.size().1 {
            self.scroll();
            self.y_pos = self.size().1;
        }
        self.x_pos = 0;
    }

    pub fn prev_char(&mut self) {
        self.x_pos -= 1;
        if self.x_pos < 0 {
            self.x_pos = self.size().0 - 1;
            self.y_pos -= 1;
        }
    }

    pub fn scroll(&self) {
        get_display_for_sure().scrollup(
            Some(self.background),
            FONT_Y,
            (FONT_Y as isize * (TOP_PAD_LINE_NUM - 1)) as usize,
        );
    }

    pub fn write_char_at(&mut self, x: isize, y: isize, c: char) {
        let mut buf = [0u8; 2];
        let str_c = c.encode_utf8(&mut buf);
        let pos = Point::new(
            x as i32 * (FONT_X + SPACING) as i32,
            (y + TOP_PAD_LINE_NUM) as i32 * FONT_Y as i32,
        );
        let mut style = MonoTextStyle::new(FONT, self.frontground);
        CharacterStyle::set_background_color(&mut style, Some(self.background));
        Text::new(str_c, pos, style)
            .draw(&mut *get_display_for_sure())
            .expect("Writing Error!");
    }

    pub fn write_char(&mut self, c: char) {
        self.write_char_at(self.x_pos, self.y_pos, c);
        self.x_pos += 1;
        if self.x_pos >= self.size().0 {
            self.next_row()
        }
    }

    pub fn write(&mut self, s: &str) {
        let mut skip = 0;
        for (idx, c) in s.chars().enumerate() {
            if idx < skip {
                continue;
            }

            match c {
                '\n' => self.next_row(),
                '\r' => self.x_pos = 0,
                '\x08' => {
                    self.prev_char();
                    self.write("  ");
                    self.prev_char();
                    self.prev_char();
                }
                // handle other control characters here
                '\x1b' => {
                    let count = self.handle_ctlseqs(s.split_at(idx + 1).1);
                    skip = idx + count + 1;
                }
                _ => self.write_char(c),
            }
        }
    }

    pub fn handle_ctlseqs(&mut self, s: &str) -> usize {
        // support list:
        // CSI n A
        // CSI n B
        // CSI n C
        // CSI n D
        // CSI y ; x H
        // CSI n J

        if !s.starts_with('[') {
            return 0;
        }

        let mut count = 1;
        let mut nums = Vec::new();
        let mut num = 0;
        for c in s.chars().skip(1) {
            count += 1;

            match c {
                '0'..='9' => {
                    num *= 10;
                    num += c as usize - '0' as usize;

                    if num > 32767 {
                        num = 32767;
                    }
                    continue;
                }
                ';' => {
                    nums.push(num);
                    num = 0;
                    continue;
                }
                _ => {
                    nums.push(num);
                }
            }

            let n = *nums.first().unwrap_or(&0) as isize;

            match c {
                'A' => {
                    self.move_cursor(0, -n);
                    break;
                }
                'B' => {
                    self.move_cursor(0, n);
                    break;
                }
                'C' => {
                    self.move_cursor(n, 0);
                    break;
                }
                'D' => {
                    self.move_cursor(-n, 0);
                    break;
                }
                'H' => {
                    let x = *nums.get(1).unwrap_or(&1) as isize;
                    self.set_cursor(x - 1, n - 1);
                    break;
                }
                'J' => {
                    if n == 2 {
                        self.clear();
                        break;
                    } else {
                        // not support
                        return 0;
                    }
                }
                _ => return 0,
            }
        }

        count
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        self.x_pos = (self.x_pos + dx).max(0).min(self.size().0 - 1);
        self.y_pos = (self.y_pos + dy).max(0).min(self.size().1 - 1);
    }

    pub fn set_cursor(&mut self, x: isize, y: isize) {
        self.x_pos = x.max(0).min(self.size().0 - 1);
        self.y_pos = y.max(0).min(self.size().1 - 1);
    }

    pub fn draw_hint(&mut self) {
        let mut buf = [0u8; 2];
        let str_c = '_'.encode_utf8(&mut buf);
        let pos = Point::new(
            self.x_pos as i32 * (FONT_X + SPACING) as i32,
            (self.y_pos + TOP_PAD_LINE_NUM) as i32 * FONT_Y as i32,
        );
        let mut style = MonoTextStyle::new(FONT, colors::GREY);
        CharacterStyle::set_background_color(&mut style, Some(self.background));
        Text::new(str_c, pos, style)
            .draw(&mut *get_display_for_sure())
            .expect("Writing Error!");
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
            FONT_Y as usize * (TOP_PAD_LINE_NUM - 1) as usize,
        );
    }

    pub fn header(&self) {
        let mut style = MonoTextStyle::new(&font::JBMONO_TITLE, colors::BLUE);
        CharacterStyle::set_background_color(&mut style, Some(colors::BACKGROUND));
        Text::with_baseline(
            crate::utils::get_header(),
            Point::new(6, 6),
            style,
            Baseline::Top,
        )
        .draw(&mut *get_display_for_sure())
        .expect("Drawing Error!");
    }
}

impl Device<u8> for Console {
    fn read(&self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize, DeviceError> {
        if offset + size >= buf.len() {
            return Err(DeviceError::ReadError);
        }
        // TODO: get key
        Ok(0)
    }

    fn write(&mut self, buf: &[u8], offset: usize, size: usize) -> Result<usize, DeviceError> {
        if let Ok(s) = core::str::from_utf8(&buf[offset..offset + size]) {
            self.write(s);
            Ok(size)
        } else {
            Err(DeviceError::WriteError)
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}

pub fn backspace() {
    let mut console = get_console_for_sure();
    console.prev_char();
    console.write("  ");
    console.prev_char();
    console.prev_char();
}
