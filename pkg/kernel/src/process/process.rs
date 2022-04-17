use super::ProgramStatus;
use crate::mem::BootInfoFrameAllocator;
use crate::utils::{Registers, RegistersValue};
use crate::mem::physical_to_virtual;
use core::intrinsics::copy_nonoverlapping;
use x86_64::structures::paging::{OffsetPageTable, PhysFrame, PageTable};
use x86_64::structures::idt::{InterruptStackFrameValue, InterruptStackFrame};
use x86_64::structures::paging::FrameAllocator;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::rflags::RFlags;
use alloc::string::String;
use alloc::vec::Vec;
use x86_64::VirtAddr;

#[derive(Debug)]
pub struct Process {
    pid: u16,
    regs: RegistersValue,
    name: String,
    parent: u16,
    status: ProgramStatus,
    priority: usize,
    ticks: usize,
    ticks_passed: usize,
    children: Vec::<u16>,
    stack_frame: InterruptStackFrameValue,
    page_table_addr: (PhysFrame, Cr3Flags),
    page_table: Option<OffsetPageTable<'static>>
}

const PRIORITY_FACTOR: usize = 5;

impl Process {
    pub fn pid(&self) -> u16 {
        self.pid
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn tick(&mut self) -> bool {
        self.ticks -= 1;
        self.ticks_passed += 1;
        self.ticks > 0
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.ticks = self.priority * PRIORITY_FACTOR;
        self.status = ProgramStatus::Running;
    }

    pub fn set_page_table_with_cr3(&mut self) {
        self.page_table_addr = Cr3::read();
    }

    pub fn is_running(&self) -> bool {
        self.status == ProgramStatus::Running
    }

    pub fn save(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        self.regs = unsafe{ regs.as_mut().read().clone() };
        self.stack_frame = unsafe{ sf.as_mut().read().clone() };
        self.status = ProgramStatus::Ready;
    }

    pub fn restore(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        unsafe {
            regs.as_mut().write(self.regs);
            sf.as_mut().write(self.stack_frame);
            Cr3::write(self.page_table_addr.0, self.page_table_addr.1)
        }
        self.ticks = self.priority * PRIORITY_FACTOR;
        self.status = ProgramStatus::Running;
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.stack_frame.stack_pointer = stack_top;
        self.stack_frame.instruction_pointer = entry;
        self.stack_frame.cpu_flags = (RFlags::IOPL_HIGH | RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits();
    }
}

impl Process {
    pub fn new(
        frame_alloc: &mut BootInfoFrameAllocator,
        pid: u16, name: String, priority: usize, parent: u16
    ) -> Self {
        // 1. alloc a page table for process
        let page_table_addr = frame_alloc.allocate_frame()
            .expect("Cannot alloc page table for new process.");
        trace!("Alloc page table for {}: {:?}", name, page_table_addr);

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
                VirtAddr::new_truncate(crate::mem::PHYSICAL_OFFSET as u64),
            )
        };

        // 4. create context
        let status = ProgramStatus::Created;
        let stack_frame = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new_truncate(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new_truncate(0),
            stack_segment: 0,
        };
        let regs = RegistersValue::default();
        let ticks = priority * PRIORITY_FACTOR;
        let ticks_passed = 0;

        debug!("New process {}#{} created.", name, pid);

        // 3. create process object
        Self {
            pid,
            name,
            parent,
            priority,
            status,
            ticks,
            ticks_passed,
            stack_frame,
            regs,
            page_table_addr: (page_table_addr, Cr3::read().1),
            page_table: Some(page_table),
            children: Vec::new()
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // TODO: deallocate memory
    }
}
