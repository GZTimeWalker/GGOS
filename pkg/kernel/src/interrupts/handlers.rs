use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(clock_handler)
        .set_stack_index(crate::gdt::CONTEXT_SWITCH);
}

pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
    crate::utils::draw::clock();
    // manager::switch_process(sf, regs);
    super::ack(consts::Interrupts::IRQ0 as u8);
}
