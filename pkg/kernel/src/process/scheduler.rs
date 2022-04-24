use x86_64::structures::idt::InterruptStackFrame;
use x86_64::VirtAddr;
use x86_64::structures::paging::FrameAllocator;
use crate::utils::Registers;
use crate::memory::*;
use super::manager::get_process_manager_for_sure;
use alloc::string::String;

pub fn switch(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    let mut manager = get_process_manager_for_sure();

    if manager.tick() {
        return;
    }

    manager.save_current(regs, sf);
    manager.switch_next(regs, sf);
}

pub fn spawn_kernel_thread(entry: fn(), name: String, priority: usize) {
    let entry = VirtAddr::new(entry as u64);

    let stack = get_frame_alloc_for_sure().allocate_frame().expect("Failed to allocate stack for kernel thread");

    let stack_top = VirtAddr::new(
        physical_to_virtual(
            stack.start_address().as_u64() as usize
        ) as u64
    );

    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();
        manager.spawn(entry, stack_top, name, priority, 0);
    });
}
