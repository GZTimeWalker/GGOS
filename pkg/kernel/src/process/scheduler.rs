use x86_64::structures::idt::InterruptStackFrame;
use crate::utils::Registers;

pub fn switch(regs: &mut Registers, sf: &mut InterruptStackFrame)
{
    x86_64::instructions::interrupts::without_interrupts(|| {
        // TODO
    })
}
