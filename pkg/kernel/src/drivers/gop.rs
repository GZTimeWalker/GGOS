use crate::utils::*;
use boot::GraphicInfo;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

#[derive(Debug)]
pub enum DisplayError {
    OutOfBounds(usize, usize),
}

pub struct GOPDisplay<'a> {
    info: &'a GraphicInfo,
    buffer: &'a mut [u32],
}

impl OriginDimensions for GOPDisplay<'_> {
    fn size(&self) -> Size {
        let (x, y) = self.info.mode.resolution();
        Size::new(x as u32, y as u32)
    }
}

impl DrawTarget for GOPDisplay<'_> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            self.draw_pixel(coord, color).unwrap();
        }
        Ok(())
    }
}

impl<'a> GOPDisplay<'a> {
    pub fn new(graphic: &'a GraphicInfo) -> Self {
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                graphic.fb_addr as *mut u32,
                graphic.mode.resolution().0 * graphic.mode.resolution().1,
            )
        };

        Self {
            info: graphic,
            buffer,
        }
    }

    pub fn draw_pixel_u32(&mut self, point: Point, color: u32) -> Result<(), DisplayError> {
        let size = self.resolution();
        let (x, y) = (point.x as usize, point.y as usize);

        if x >= size.0 || y >= size.1 {
            return Err(DisplayError::OutOfBounds(x, y));
        }

        let index = y * size.0 + x;
        self.buffer[index] = color;

        Ok(())
    }

    pub fn draw_pixel(
        &mut self,
        point: Point,
        color: <GOPDisplay<'a> as DrawTarget>::Color,
    ) -> Result<(), DisplayError> {
        self.draw_pixel_u32(point, color.into_storage())
    }

    pub fn resolution(&self) -> (usize, usize) {
        self.info.mode.resolution()
    }

    pub fn clear(&mut self, color: Option<<GOPDisplay<'a> as DrawTarget>::Color>, base: usize) {
        let size = self.resolution();
        let color = color.unwrap_or(colors::BACKGROUND);
        let buf = self.buffer.as_mut_ptr();

        unsafe {
            // if the color is purely grey, set the buffer with bytes
            if color.r() == color.g() && color.g() == color.b() {
                core::ptr::write_bytes(
                    buf.offset((base as isize) * size.0 as isize),
                    color.r(),
                    (size.1 - base) * size.0,
                );
            } else {
                let color = color.into_storage();
                for idx in base * size.0..size.1 * size.0 {
                    self.buffer[idx] = color;
                }
            }
        }
    }

    pub fn scrollup(
        &mut self,
        color: Option<<GOPDisplay<'a> as DrawTarget>::Color>,
        n: u8,
        base: usize,
    ) {
        let size = self.resolution();
        let color = color.unwrap_or_default();
        let buf = self.buffer.as_mut_ptr();
        let n = n as isize;

        unsafe {
            core::ptr::copy(
                buf.offset((base as isize + n) * size.0 as isize) as *const u32,
                buf.offset(base as isize * size.0 as isize),
                (size.1 - base) * size.0,
            );
            // if the color is purely grey, set the buffer with bytes
            if color.r() == color.g() && color.g() == color.b() {
                core::ptr::write_bytes(
                    buf.offset((size.1 as isize - n) * size.0 as isize),
                    color.r(),
                    n as usize * size.0,
                );
            } else {
                let color = color.into_storage();
                for idx in (size.1 - n as usize) * size.0..size.1 * size.0 {
                    self.buffer[idx] = color;
                }
            }
        }
    }
}
