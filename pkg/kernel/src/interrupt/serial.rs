use super::consts;
use crate::{drivers::serial::get_serial_for_sure, push_key};
use alloc::vec;
use pc_keyboard::DecodedKey;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[consts::Interrupts::IrqBase as u8 + consts::Irq::Serial0 as u8]
        .set_handler_fn(interrupt_handler);
}

pub fn init() {
    super::enable_irq(consts::Irq::Serial0 as u8, 0);
    debug!("Serial0(COM1) IRQ enabled.");
}

/// Receive character from uart 16550
/// Should be called on every interrupt
pub fn receive() {
    let mut buf = vec::Vec::with_capacity(4);
    while let Some(scancode) = get_serial_for_sure().receive() {
        match scancode {
            127 => push_key(DecodedKey::Unicode('\x08')),
            13 => push_key(DecodedKey::Unicode('\n')),
            c => {
                buf.push(c);

                if let Ok(s) = core::str::from_utf8(&buf) {
                    let chr = s.chars().next().unwrap();
                    push_key(DecodedKey::Unicode(chr));
                    buf.clear();
                }
            }
        }
    }
}

pub extern "x86-interrupt" fn interrupt_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}
