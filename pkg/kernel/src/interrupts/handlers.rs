use crate::utils::Registers;
use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(clock_handler)
        .set_stack_index(crate::gdt::CONTEXT_SWITCH);
}

pub extern "C" fn clock(mut regs: Registers, mut sf: InterruptStackFrame ) {
    crate::process::switch( &mut regs, &mut sf);
    crate::utils::draw::clock();
    super::ack(consts::Interrupts::IRQ0 as u8);
}

as_handler!(clock);
