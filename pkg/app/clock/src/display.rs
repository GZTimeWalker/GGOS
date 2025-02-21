use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{Dimensions, IntoStorage, Point};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{Pixel, draw_target::DrawTarget, prelude::Size};
use lib::*;

pub struct SysDisplay;

impl Dimensions for SysDisplay {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(1280, 800)) // TODO: get from kernel
    }
}

impl DrawTarget for SysDisplay {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(coord, color) = pixel;

            sys_draw(
                if coord.x < 0 { 0 } else { coord.x },
                if coord.y < 0 { 0 } else { coord.y },
                color.into_storage(),
            );
        }

        Ok(())
    }
}
