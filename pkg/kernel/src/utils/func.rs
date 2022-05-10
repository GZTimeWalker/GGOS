// use crate::display::get_display_for_sure;
// use crate::interrupt::*;

pub fn test() -> ! {
    let mut count = 0;
    let id;
    if let Some(id_env) = crate::process::env("id") {
        id = id_env
    } else {
        id = "unknown".into()
    }
    loop {
        count += 1;
        if count == 100 {
            count = 0;
            print_serial!("\r{:-6} => Hello, world!", id);
        }
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// #[no_mangle]
// pub extern "C" fn syscall(syscall: usize, arg0: usize, arg1: usize, arg2: usize) {
//     unsafe {
//         core::arch::asm!(
//             "mov rbx, {}
//              int 0x80",
//             in(reg) arg0,
//             in("rax") syscall,
//             in("rcx") arg1,
//             in("rdx") arg2
//         );
//     }
// }

// pub fn clock() -> ! {
//     let mut angle: f32 = 90.0;
//     const ANGLE_INCR: f32 = 15.0;
//     const D_OFFSET: i32 = 4;
//     let cx = x86_64::instructions::interrupts::without_interrupts(|| {
//         get_display_for_sure().resolution().0
//     });

//     use crate::utils::colors;
//     use embedded_graphics::prelude::*;
//     use embedded_graphics::primitives::*;
//     #[allow(unused_imports)]
//     use micromath::F32Ext;

//     loop {
//         let mut start: i64 = 0;

//         syscall(
//             Syscall::Clock as usize,
//             (&mut start as *mut i64) as usize,
//             0,
//             0,
//         );

//         let mut current = start;

//         while current - start < 1000_0000 {
//             syscall(
//                 Syscall::Clock as usize,
//                 (&mut current as *mut i64) as usize,
//                 0,
//                 0,
//             );
//         }

//         angle += ANGLE_INCR;
//         if angle >= 360.0 {
//             angle = 0.0;
//         }
//         let value = angle / 180f32 * core::f32::consts::PI;

//         let len = 24i32;
//         let (cx, cy) = (cx as i32 - len - 10, len + 8);

//         let (dx, dy) = (
//             (len as f32 * value.cos()) as i32,
//             (len as f32 * value.sin()) as i32,
//         );

//         x86_64::instructions::interrupts::without_interrupts(|| {
//             if let Some(mut display) = crate::drivers::display::get_display() {
//                 Circle::new(
//                     Point::new(cx - len - D_OFFSET, cy - len - D_OFFSET),
//                     (2 * len + D_OFFSET * 2) as u32,
//                 )
//                 .into_styled(PrimitiveStyle::with_fill(colors::FRONTGROUND))
//                 .draw(&mut *display)
//                 .unwrap();

//                 Line::new(Point::new(cx, cy), Point::new(cx + dx, cy + dy))
//                     .into_styled(PrimitiveStyle::with_stroke(colors::GREEN, 5))
//                     .draw(&mut *display)
//                     .unwrap();
//             }
//         });
//     }
// }
