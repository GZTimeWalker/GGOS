use super::consts;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode
};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(unsafe {
            core::mem::transmute(clock_handler as *mut fn())
        }).set_stack_index(crate::gdt::CONTEXT_SWITCH);
}

#[repr(align(8), C)]
#[derive(Debug, Clone, Default)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rbp: usize,
}

pub extern "x86-interrupt" fn clock_handler(sf: &mut InterruptStackFrame, regs: &mut Registers) {
    //crate::process::switch_first_ready_process(sf, regs);
    clock_draw();
    super::ack(consts::Interrupts::IRQ0 as u8);
}

fn clock_draw() {
    static ANGLE: spin::Mutex<u16> = spin::Mutex::new(90);
    const ANGLE_INCR: u16 = 15;

    x86_64::instructions::interrupts::without_interrupts(|| {
        use embedded_graphics::pixelcolor::*;
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
        } else {
            value = 0.0;
        }

        let (cx, cy) = (8 * 79, 16 * 24);
        let len = 16u32;

        #[allow(unused_imports)]
        use micromath::F32Ext;
        let (dx, dy) = (
            (len as f32 * value.cos()) as i32,
            (len as f32 * value.sin()) as i32,
        );

        if let Some(mut display) = crate::display::get_display() {
            Circle::new(Point::new(cx, cy), len)
                .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                .draw(&mut *display)
                .unwrap(); // FIXME: report error later

            Line::new(Point::new(cx, cy), Point::new(cx - dx, cy - dy))
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 3))
                .draw(&mut *display)
                .unwrap(); // FIXME: report error later
        }
    })
}
