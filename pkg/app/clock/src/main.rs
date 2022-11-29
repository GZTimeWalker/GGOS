#![no_std]
#![no_main]
#![allow(unreachable_code)]

use chrono::Duration;

use embedded_graphics::pixelcolor::Rgb888;
use lib::*;

mod display;

extern crate lib;

fn main() -> ! {
    clock();
}

fn clock() -> ! {
    let mut angle: f32 = 90.0;
    const ANGLE_INCR: f32 = 15.0;
    const D_OFFSET: i32 = 4;
    let cx = 1280; // TODO: read from kernel

    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::*;
    #[allow(unused_imports)]
    use micromath::F32Ext;

    loop {
        let start = sys_time();
        let mut current = start;

        while (current - start) < Duration::seconds(1) {
            current = sys_time();
        }

        angle += ANGLE_INCR;
        if angle >= 360.0 {
            angle = 0.0;
        }
        let value = angle / 180f32 * core::f32::consts::PI;

        let len = 24i32;
        let (cx, cy) = (cx as i32 - len - 10, len + 8);

        let (dx, dy) = (
            (len as f32 * value.cos()) as i32,
            (len as f32 * value.sin()) as i32,
        );

        Circle::new(
            Point::new(cx - len - D_OFFSET, cy - len - D_OFFSET),
            (2 * len + D_OFFSET * 2) as u32,
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
        .draw(&mut display::SysDisplay)
        .unwrap();

        Line::new(Point::new(cx, cy), Point::new(cx + dx, cy + dy))
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb888::new(0x32, 0x8e, 0x2e),
                5,
            ))
            .draw(&mut display::SysDisplay)
            .unwrap();
    }
}

entry!(main);
