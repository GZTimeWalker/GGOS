use crate::keyboard::get_keyboard_for_sure;
use crate::{interrupt::consts::*, push_key};
use pc_keyboard::DecodedKey;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Keyboard as u8].set_handler_fn(interrupt_handler);
}

pub fn init() {
    super::enable_irq(Irq::Keyboard as u8, 0);
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
    if let Some(key) = receive() {
        push_key(key);
    }
    super::ack();
}
