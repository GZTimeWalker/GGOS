use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pc_keyboard::DecodedKey;
use crate::drivers::{
    input::get_input_buf_for_sure,
    serial::get_serial_for_sure
};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Serial0 as u8) as usize]
        .set_handler_fn(interrupt_handler);
}

pub fn init() {
    super::enable_irq(consts::IRQ::Serial0 as u8);
    debug!("Serial0(COM1) IRQ enabled.");
}

/// Receive character from uart 16550
/// Should be called on every interrupt
pub fn receive() -> Option<DecodedKey> {

    if let Some(scancode) = get_serial_for_sure().receive_no_wait() {
        return match scancode {
            127 => Some(DecodedKey::Unicode('\x08')),
            13 => Some(DecodedKey::Unicode('\n')),
            c => Some(DecodedKey::Unicode(c as char))
        };
    }

    None
}

pub extern "x86-interrupt" fn interrupt_handler(_st: InterruptStackFrame) {
    super::ack(super::consts::IRQ::Serial0 as u8);
    if let Some(key) = receive() {
        get_input_buf_for_sure().push(key).unwrap();
    }
}
