use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pc_keyboard::DecodedKey;
use crate::drivers::{
    input::get_input_buf_for_sure,
    serial::get_serial_for_sure,
    keyboard::get_keyboard_for_sure
};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::COM1 as u8) as usize]
        .set_handler_fn(interrupt_handler);
}

pub fn init() {
    super::enable_irq(consts::IRQ::COM1 as u8);
    debug!("COM1 IRQ enabled");
}

/// Receive character from uart 16550
/// Should be called on every interrupt
pub fn receive() -> Option<DecodedKey> {

    if let Some(scancode) = get_serial_for_sure().receive_no_wait() {
        let mut keyboard = get_keyboard_for_sure();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            return keyboard.process_keyevent(key_event);
        }
    }

    None
}

pub extern "x86-interrupt" fn interrupt_handler(_st: InterruptStackFrame) {
    super::ack(super::consts::IRQ::COM1 as u8);
    println_console!("UART INT");
    if let Some(key) = receive() {
        get_input_buf_for_sure().push(key).unwrap();
    }
}
