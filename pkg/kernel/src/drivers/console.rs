#![allow(dead_code)]

use core::fmt::Write;
use crate::utils::font;
use crate::utils::colors;
use crate::drivers::display::get_display_for_sure;
use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{renderer::CharacterStyle, Baseline, Text},
};

once_mutex!(pub CONSOLE: Console);

const FONT: &MonoFont = &font::JBMONO;

const FONT_X: u8 = FONT.character_size.width as u8;
const FONT_Y: u8 = FONT.character_size.height as u8;
const SPACING: u8 = FONT.character_spacing as u8;

const TOP_PAD_LINE_NUM: usize = 3;

pub fn initialize() {
    init_CONSOLE(Console::new());
    let console = get_console_for_sure();
    console.clear();
    console.header();
}

guard_access_fn!(pub get_console(CONSOLE: Console));

pub struct Console {
    x_pos: usize,
    y_pos: usize,
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

impl Console {
    pub fn size(&self) -> (usize, usize) {
        let size: Size = get_display_for_sure().size();
        (
            size.width as usize / (FONT_X + SPACING) as usize,
            size.height as usize / FONT_Y as usize - TOP_PAD_LINE_NUM,
        )
    }

    fn get_char_pos(&self, x: usize, y: usize) -> (usize, usize) {
        (
            x * FONT_X as usize,
            (y + TOP_PAD_LINE_NUM) * FONT_Y as usize,
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

    pub fn next_char(&mut self) {
        self.x_pos += 1;
        if self.x_pos >= self.size().0 {
            self.next_row()
        }
    }

    pub fn scroll(&self) {
        get_display_for_sure().scrollup(
            Some(self.background),
            FONT_Y,
            FONT_Y as usize * (TOP_PAD_LINE_NUM - 1),
        );
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char) {
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

    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.next_row();
                }
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
        )
        .into_styled(PrimitiveStyle::with_stroke(colors::FRONTGROUND, 2))
        .draw(&mut *get_display_for_sure())
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
        get_display_for_sure().clear(Some(self.background), FONT_Y as usize * TOP_PAD_LINE_NUM);
    }

    pub fn header(&self) {
        let mut style = MonoTextStyle::new(&font::JBMONO_TITLE, colors::BLUE);
        CharacterStyle::set_background_color(&mut style, Some(colors::BACKGROUND));
        Text::with_baseline(crate::utils::HEADER, Point::new(6, 6), style, Baseline::Top)
            .draw(&mut *get_display_for_sure())
            .expect("Drawing Error!");
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
