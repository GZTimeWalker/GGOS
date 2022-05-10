use x86_64::VirtAddr;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::FrameAllocator;
use alloc::string::String;
use crate::memory::*;
use crate::utils::Registers;
use crate::process::ProcessData;
use super::manager::get_process_manager_for_sure;

pub fn switch(regs: &mut Registers, sf: &mut InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut manager = get_process_manager_for_sure();

        manager.save_current(regs, sf);
        manager.switch_next(regs, sf);
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as u64);

        let stack = get_frame_alloc_for_sure().allocate_frame()
            .expect("Failed to allocate stack for kernel thread");

        let stack_top = VirtAddr::new(physical_to_virtual(
            stack.start_address().as_u64()) + FRAME_SIZE);

        let mut manager = get_process_manager_for_sure();
        manager.spawn_kernel_thread(entry, stack_top, name, 0, data);
    });
}
