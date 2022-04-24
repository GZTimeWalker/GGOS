pub fn test(id: usize) -> ! {
    let mut count = 0;
    loop {
        count += 1;
        if count == 100 {
            count = 0;
            trace!("[{}] Hello, world!", id);
        }
    }
}

pub fn clock() -> ! {
    let mut angle: f32 = 90.0;
    const ANGLE_INCR: f32 = 1.0;
    const D_OFFSET: i32 = 4;
    let (cx, _) = crate::drivers::display::get_display().unwrap().resolution();

    use crate::utils::colors;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::*;
    #[allow(unused_imports)]
    use micromath::F32Ext;

    loop {
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

        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(mut display) = crate::drivers::display::get_display() {
                Circle::new(
                    Point::new(cx - len - D_OFFSET, cy - len - D_OFFSET),
                    (2 * len + D_OFFSET * 2) as u32,
                )
                .into_styled(PrimitiveStyle::with_fill(colors::FRONTGROUND))
                .draw(&mut *display)
                .unwrap();

                Line::new(Point::new(cx, cy), Point::new(cx + dx, cy + dy))
                    .into_styled(PrimitiveStyle::with_stroke(colors::GREEN, 5))
                    .draw(&mut *display)
                    .unwrap();
            }
        })
    }
}
