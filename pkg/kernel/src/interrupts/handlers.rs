use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(core::mem::transmute(clock_handler as *mut fn()))
        .set_stack_index(crate::gdt::CONTEXT_SWITCH);
}

pub extern "x86-interrupt" fn clock_handler(_sf: &mut InterruptStackFrame) {
    crate::utils::draw::clock();
    super::ack(consts::Interrupts::IRQ0 as u8);
}
