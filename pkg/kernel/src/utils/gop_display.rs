use boot::GraphicInfo;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;

const PIXEL_LEN: usize = 1;

#[derive(Debug)]
pub enum DisplayError {
    OutOfBounds(usize, usize),
}

pub struct GOPDisplay<'a>{
    info: &'a GraphicInfo,
    buffer: &'a mut [u32]
}

impl <'a> OriginDimensions for GOPDisplay<'a> {
    fn size(&self) -> Size {
        let (x, y) = self.info.mode.resolution();
        Size::new((x / PIXEL_LEN) as u32, (y / PIXEL_LEN) as u32)
    }
}

impl<'a> DrawTarget for GOPDisplay<'a> {

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

impl <'a> GOPDisplay <'a> {

    pub fn new(graphic: &'a GraphicInfo) -> Self {
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                graphic.fb_addr as *mut u32,
                graphic.mode.resolution().0 as usize * graphic.mode.resolution().1 as usize
            )
        };

        Self {
            info: graphic,
            buffer
        }
    }

    pub fn draw_pixel(&mut self, point: Point, color: <GOPDisplay<'a> as DrawTarget>::Color) -> Result<(), DisplayError> {

        let size = self.resolution();
        let (x, y) = (point.x as usize, point.y as usize);

        if x >= size.0 || y >= size.1 {
            return Err(DisplayError::OutOfBounds(x, y));
        }

        let color = color.into_storage();

        for dx in 0..PIXEL_LEN {
            for dy in 0..PIXEL_LEN {
                let index = (y + dy) * size.0 + x + dx;
                self.buffer[index] = color;
            }
        }

        Ok(())
    }

    pub fn resolution(&self) -> (usize, usize) {
        self.info.mode.resolution()
    }

    pub fn clear(&mut self, color: Option<<GOPDisplay<'a> as DrawTarget>::Color>) {
        let size = self.resolution();
        let color = color.unwrap_or_default().into_storage();

        for index in 0..(size.0 * size.1) {
            self.buffer[index] = color;
        }
    }

    pub fn scrollup(&mut self, color: Option<<GOPDisplay<'a> as DrawTarget>::Color>, n: u8) {
        let size = self.resolution();
        let color = color.unwrap_or_default().into_storage();

        for y in 0..size.1 {
            for x in 0..size.0 {
                let index = (y * size.0 + x) as usize;
                let index_up = (y as isize - n as isize) * size.0 as isize + x as isize;
                if index_up >= 0 {
                    self.buffer[index_up as usize] = self.buffer[index];
                }
                self.buffer[index] = color;
            }
        }
    }
}
