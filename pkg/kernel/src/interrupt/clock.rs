use super::consts;
use crate::{memory::gdt, proc::ProcessContext};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    unsafe {
        idt[consts::Interrupts::IrqBase as u8 + consts::Irq::Timer as u8]
            .set_handler_fn(clock_handler)
            .set_stack_index(gdt::CONTEXT_SWITCH_IST_INDEX);
    }
}

pub extern "C" fn clock(mut context: ProcessContext) {
    crate::proc::switch(&mut context);
    super::ack();
}

as_handler!(clock);
