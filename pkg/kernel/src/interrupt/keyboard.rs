use super::consts;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};
use pc_keyboard::DecodedKey;
use crate::{keyboard::get_keyboard_for_sure, push_key};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::Irq0 as u8 + consts::Irq::Keyboard as u8) as usize]
        .set_handler_fn(interrupt_handler);
}

pub fn init() {
    super::enable_irq(consts::Irq::Keyboard as u8);
    debug!("Keyboard IRQ enabled.");
}

/// Receive character from keyboard
/// Should be called on every interrupt
pub fn receive() -> Option<DecodedKey> {

    let mut keyboard = get_keyboard_for_sure();
    let mut data_port = Port::<u8>::new(0x60);
    let mut status_port = Port::<u8>::new(0x64);

    // Output buffer status = 1
    if unsafe { status_port.read() } & 0x1 != 0 {
        let scancode = unsafe { data_port.read() };
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            return keyboard.process_keyevent(key_event);
        }
    }

    None
}

pub extern "x86-interrupt" fn interrupt_handler(_st: InterruptStackFrame) {
    super::ack(super::consts::Irq::Keyboard as u8);
    if let Some(key) = receive() {
        push_key(key);
    }
}
