pub fn test() -> ! {
    let mut count = 0;
    let id;
    if let Some(id_env) = crate::process::env("id") {
        id = id_env
    } else { id = "unknown".into()}
    loop {
        count += 1;
        if count == 100 {
            count = 0;
            print_serial!("\r{:-6} => Hello, world!", id);
        }
        unsafe {
           core::arch::asm!("hlt")
        }
    }
}

pub fn clock() -> ! {
    let mut angle: f32 = 90.0;
    const ANGLE_INCR: f32 = 15.0;
    const D_OFFSET: i32 = 4;

    let cx = match crate::drivers::display::get_display() {
        Some(display) => display.resolution().0,
        None => 1024
    };

    use crate::utils::colors;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::*;
    #[allow(unused_imports)]
    use micromath::F32Ext;

    loop {
        super::clock::spin_wait_for_ns(10000);

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
        });
    }
}
