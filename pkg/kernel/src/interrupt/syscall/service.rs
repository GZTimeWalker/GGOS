use crate::{utils::*, display::get_display_for_sure};
use embedded_graphics::prelude::*;

pub fn sys_clock() -> i64 {
    clock::now().timestamp_nanos()
}

pub fn sys_draw(x: usize, y: usize, color: usize) {
    let _ = get_display_for_sure().draw_pixel_u32(
        Point::new(x as i32, y as i32),
        color as u32
    );
}
