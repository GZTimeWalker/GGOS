pub fn clock() {
    static ANGLE: spin::Mutex<u16> = spin::Mutex::new(90);
    const ANGLE_INCR: u16 = 1;
    const D_OFFSET: i32 = 4;

    x86_64::instructions::interrupts::without_interrupts(|| {
        use crate::utils::colors;
        use embedded_graphics::prelude::*;
        use embedded_graphics::primitives::*;

        let value;
        // 自增
        if let Some(mut angle_locked) = ANGLE.try_lock() {
            *angle_locked += ANGLE_INCR;
            if *angle_locked >= 360 {
                *angle_locked = 0;
            }
            value = *angle_locked as f32 / 180f32 * core::f32::consts::PI;
            //print!("[>] Angle = {:>3}\r", *angle_locked);
            //println!("[>] Testing......{:>3}", *angle_locked);
        } else {
            value = 0.0;
        }

        if let Some(mut display) = crate::drivers::display::get_display() {
            let len = 24i32;
            let (cx, _) = display.resolution();
            let (cx, cy) = (cx as i32 - len - 10, len + 8);

            #[allow(unused_imports)]
            use micromath::F32Ext;
            let (dx, dy) = (
                (len as f32 * value.cos()) as i32,
                (len as f32 * value.sin()) as i32,
            );

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