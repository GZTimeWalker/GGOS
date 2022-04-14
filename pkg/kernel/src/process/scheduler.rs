use x86_64::structures::idt::InterruptStackFrame;
use crate::utils::Registers;

pub fn switch(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    let mut manager = super::manager::get_process_manager_for_sure();

    if manager.tick() {
        return;
    }

    manager.save_current(regs, sf);
    manager.switch_next(regs, sf);
}
