use super::ProgramStatus;
use crate::memory::BootInfoFrameAllocator;
use crate::utils::Registers;
use crate::memory::physical_to_virtual;
use core::intrinsics::copy_nonoverlapping;
use x86_64::structures::paging::{OffsetPageTable, PhysFrame, PageTable};
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::FrameAllocator;
use x86_64::registers::control::{Cr3, Cr3Flags};
use alloc::string::String;
use alloc::vec::Vec;
use x86_64::VirtAddr;

once_mutex!(pub PROCESSES: Vec<Process>);
guard_access_fn! {
    pub get_processes(PROCESSES: Vec<Process>)
}

#[derive(Debug)]
pub struct Process {
    pub pid: usize,
    pub name: String,
    pub status: ProgramStatus,
    pub priority: usize,
    pub ticks: usize,
    pub ticks_passed: usize,
    pub stack_frame: InterruptStackFrameValue,
    pub regs: Registers,
    pub page_table_addr: (PhysFrame, Cr3Flags),
    pub page_table: Option<OffsetPageTable<'static>>
}


impl Process {
    pub fn new(
        frame_alloc: &mut BootInfoFrameAllocator,
        pid: usize, name: String, priority: usize
    ) -> Self {
        // 1. alloc a page table for process
        let page_table_addr = frame_alloc.allocate_frame()
            .expect("Cannot alloc page table for new process.");
        trace!("Alloc page table for new process: {:?}", page_table_addr);

        // 2. copy current page table to new page table
        unsafe {
            copy_nonoverlapping::<PageTable>(
                Cr3::read().0.start_address().as_u64() as *mut PageTable,
                page_table_addr.start_address().as_u64() as *mut PageTable,
                1
            );
        }

        // 3. create page table object
        let page_table_raw = unsafe {
            (physical_to_virtual(page_table_addr.start_address().as_u64() as usize)
                as *mut PageTable)
            .as_mut()
        }.unwrap();

        let page_table = unsafe {
            OffsetPageTable::new(
                page_table_raw,
                VirtAddr::new_truncate(crate::memory::PHYSICAL_OFFSET as u64),
            )
        };

        // 4. create context
        let status = ProgramStatus::Creating;
        let stack_frame = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new_truncate(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new_truncate(0),
            stack_segment: 0,
        };
        let regs = Registers::default();
        let ticks = priority * 10;
        let ticks_passed = 0;

        debug!("New process {}#{} created.", name, pid);

        // 3. create process object
        Self {
            pid,
            name,
            priority,
            status,
            ticks,
            ticks_passed,
            stack_frame,
            regs,
            page_table_addr: (page_table_addr, Cr3::read().1),
            page_table: Some(page_table),
        }
    }
}
