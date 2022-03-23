#![no_std]
#![no_main]

use boot::{BootInfo, GraphicInfo};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::*;
use embedded_graphics::text::*;
use core::panic::PanicInfo;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::Pixel;

mod color_line;

const PIXEL_LEN: i32 = 2;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    let mode = boot_info.graphic_info.mode;

    let (display_x, display_y) = mode.resolution();

    let fb_addr = boot_info.graphic_info.fb_addr;

    for i in 0..(display_x * display_y) as isize {
        unsafe {
            *(fb_addr as *mut u32).offset(i).as_mut().unwrap() = 0x00202020;
        }
    }

    let (display_x, display_y) = ((display_x / 2) as i32, (display_y / 2) as i32);
    let mut display = GOPDisplay(&boot_info.graphic_info);

    use embedded_graphics::mono_font::ascii::FONT_9X18;

    let style = MonoTextStyle::new(&FONT_9X18, Rgb888::WHITE);

    let next = Text::with_alignment(
        "Hello GGOS in Rust!",
        Point::new(display_x / 2, display_y / 2),
        style,
        Alignment::Center,
    ).draw(&mut display).expect("Draw error");

    color_line::draw(fb_addr as *mut u32, mode.resolution(), 2);

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

struct GOPDisplay<'a>(&'a GraphicInfo);

impl <'a> OriginDimensions for GOPDisplay<'a> {
    fn size(&self) -> Size {
        let (x, y) = self.0.mode.resolution();
        Size::new(x as u32, y as u32)
    }
}

impl<'a> DrawTarget for GOPDisplay<'a> {
    type Color = Rgb888;

    type Error = &'static str;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            for dx in 0..PIXEL_LEN {
                for dy in 0..PIXEL_LEN {
                    unsafe {
                        *(self.0.fb_addr as *mut u32)
                            .offset(
                                (((coord.y * PIXEL_LEN + dy) as usize) * self.0.mode.stride() +
                                    ((coord.x * PIXEL_LEN + dx) as usize)) as isize,
                            )
                            .as_mut()
                            .unwrap() =
                            (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32);
                    }
                }
            }
        }
        Ok(())
    }
}
