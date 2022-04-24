use crate::utils::Registers;
use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);

    idt.divide_error.set_handler_fn(divide_error_handler);

    idt.page_fault.set_handler_fn(page_fault_handler);

    idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);

    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(clock_handler)
        .set_stack_index(crate::gdt::CONTEXT_SWITCH);
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT, ERROR_CODE: {}\n\n{:#?}", error_code, stack_frame);
}

pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!("EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\n{:#?}", error_code, stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: STACK SEGMENT FAULT, ERROR_CODE: {}\n\n{:#?}", error_code, stack_frame);
}

pub extern "C" fn clock(mut regs: Registers, mut sf: InterruptStackFrame ) {
    crate::process::switch( &mut regs, &mut sf);
    // crate::utils::draw::clock();
    super::ack(consts::Interrupts::IRQ0 as u8);
}

as_handler!(clock);
