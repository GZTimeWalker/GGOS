use super::ProcessId;
use super::*;
use crate::filesystem::StdIO;
use crate::memory::*;
use crate::utils::{Registers, RegistersValue, Resource};
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::intrinsics::copy_nonoverlapping;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::{InterruptStackFrame, InterruptStackFrameValue};
use x86_64::structures::paging::mapper::CleanUp;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::*;
use x86_64::{PhysAddr, VirtAddr};
use xmas_elf::ElfFile;

pub struct Process {
    pid: ProcessId,
    regs: RegistersValue,
    name: String,
    parent: ProcessId,
    status: ProgramStatus,
    ticks_passed: usize,
    children: Vec<ProcessId>,
    stack_frame: InterruptStackFrameValue,
    page_table_addr: (PhysFrame, Cr3Flags),
    page_table: Option<OffsetPageTable<'static>>,
    proc_data: ProcessData,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessData {
    env: BTreeMap<String, String>,
    code_segements: Option<Vec<PageRangeInclusive>>,
    file_handles: BTreeMap<u8, Resource>,
}

impl ProcessData {
    pub fn new() -> Self {
        let env = BTreeMap::new();
        let mut file_handles = BTreeMap::new();
        // stdin, stdout, stderr
        file_handles.insert(0, Resource::Console(StdIO::Stdin));
        file_handles.insert(1, Resource::Console(StdIO::Stdout));
        file_handles.insert(2, Resource::Console(StdIO::Stderr));
        // 3 is the file self
        let code_segements = None;
        Self {
            env,
            code_segements,
            file_handles,
        }
    }

    pub fn add_file(mut self, file: &File) -> Self {
        let fd = self.file_handles.len() as u8;
        self.file_handles.insert(fd, Resource::File(file.clone()));
        self
    }

    pub fn set_env(mut self, key: &str, val: &str) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }
}

impl Process {
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn block(&mut self) {
        self.status = ProgramStatus::Blocked;
    }

    pub fn set_page_table_with_cr3(&mut self) {
        self.page_table_addr = Cr3::read();
    }

    pub fn page_table_addr(&self) -> PhysFrame {
        self.page_table_addr.0.clone()
    }

    pub fn is_running(&self) -> bool {
        self.status == ProgramStatus::Running
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.proc_data.env.get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.proc_data.env.insert(key.into(), val.into());
    }

    pub fn save(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        self.regs = unsafe { regs.as_mut().read().clone() };
        self.stack_frame = unsafe { sf.as_mut().read().clone() };
        self.status = ProgramStatus::Ready;
    }

    pub fn handle(&self, fd: u8) -> Option<Resource> {
        self.proc_data.file_handles.get(&fd).cloned()
    }

    pub fn restore(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        unsafe {
            regs.as_mut().write(self.regs);
            sf.as_mut().write(self.stack_frame);
            Cr3::write(self.page_table_addr.0, self.page_table_addr.1)
        }
        self.status = ProgramStatus::Running;
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.stack_frame.stack_pointer = stack_top;
        self.stack_frame.instruction_pointer = entry;
        self.stack_frame.cpu_flags =
            (RFlags::IOPL_HIGH | RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits();
    }

    pub fn open(&mut self, res: Resource) -> u8 {
        let fd = self.proc_data.file_handles.len() as u8;
        self.proc_data.file_handles.insert(fd, res);
        fd
    }

    pub fn close(&mut self, fd: u8) -> bool {
        self.proc_data.file_handles.remove(&fd).is_some()
    }

    fn clone_page_table(
        page_table_source: PhysFrame,
        frame_alloc: &mut BootInfoFrameAllocator,
    ) -> (OffsetPageTable<'static>, PhysFrame) {
        let page_table_addr = frame_alloc
            .allocate_frame()
            .expect("Cannot alloc page table for new process.");

        // 2. copy current page table to new page table
        unsafe {
            copy_nonoverlapping::<PageTable>(
                page_table_source.start_address().as_u64() as *mut PageTable,
                page_table_addr.start_address().as_u64() as *mut PageTable,
                1,
            );
        }

        // 3. create page table object
        let page_table_raw = unsafe {
            (physical_to_virtual(page_table_addr.start_address().as_u64()) as *mut PageTable)
                .as_mut()
        }
        .unwrap();

        let page_table = unsafe {
            OffsetPageTable::new(
                page_table_raw,
                VirtAddr::new_truncate(crate::memory::PHYSICAL_OFFSET as u64),
            )
        };

        (page_table, page_table_addr)
    }

    pub fn new(
        frame_alloc: &mut BootInfoFrameAllocator,
        name: String,
        parent: ProcessId,
        page_table_source: PhysFrame,
        proc_data: Option<ProcessData>,
    ) -> Self {
        let name = name.to_ascii_lowercase();

        // 1. create page table
        let (page_table, page_table_addr) = Self::clone_page_table(page_table_source, frame_alloc);

        trace!("Alloc page table for {}: {:?}", name, page_table_addr);

        // 2. create context
        let status = ProgramStatus::Created;
        let stack_frame = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new_truncate(0),
            code_segment: 8,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new_truncate(0),
            stack_segment: 0,
        };
        let regs = RegistersValue::default();
        let ticks_passed = 0;
        let pid = ProcessId::new();

        trace!("New process {}#{} created.", name, pid);

        // 3. create process object
        Self {
            pid,
            name,
            parent,
            status,
            ticks_passed,
            stack_frame,
            regs,
            page_table_addr: (page_table_addr, Cr3::read().1),
            page_table: Some(page_table),
            children: Vec::new(),
            proc_data: proc_data.unwrap_or_default(),
        }
    }

    pub fn add_child(&mut self, child: ProcessId) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child: ProcessId) {
        self.children.retain(|c| *c != child);
    }

    pub fn parent(&self) -> ProcessId {
        self.parent
    }

    fn clone_stack(&self, offset: u64) {
        // assume that every thread stack is the same size (STACK_PAGES * PAGE_SIZE)
        let cur_stack_start = self.stack_frame.stack_pointer.as_u64() & STACK_START_MASK;
        let offset_stack_start = STACK_BOT + offset;

        trace!(
            "Clone stack: {:#x} -> {:#x}",
            cur_stack_start,
            offset_stack_start
        );

        // copy stack
        unsafe {
            copy_nonoverlapping::<u8>(
                cur_stack_start as *mut u8,
                offset_stack_start as *mut u8,
                STACK_SIZE as usize,
            );
        }
    }

    pub fn fork(&mut self) -> Process {
        // deep clone page is not impl yet, so we put the thread stack to a new mem
        let frame_alloc = &mut *get_frame_alloc_for_sure();

        // use the same page table with the parent, but remap stack with offset
        // offset to STACK_BOT
        let mut offset = (self.children.len() as u64 + 1) * STACK_SIZE;

        while let Err(_) = elf::map_stack(
            STACK_BOT + offset,
            STACK_PAGES,
            self.page_table.as_mut().unwrap(),
            frame_alloc,
        ) {
            trace!("Map thread stack to {:#x} failed.", STACK_BOT + offset);
            offset += STACK_SIZE;
        }

        trace!("Map thread stack to {:#x} succeed.", STACK_BOT + offset);

        self.clone_stack(offset);

        let mut new_stack_frame = self.stack_frame.clone();

        // offset to current stack_start
        let offset = STACK_BOT + offset - self.stack_frame.stack_pointer.as_u64() & STACK_START_MASK;
        new_stack_frame.stack_pointer += offset + STACK_SIZE;

        let page_table_raw = unsafe {
            (physical_to_virtual(self.page_table_addr.0.start_address().as_u64()) as *mut PageTable)
                .as_mut()
        }
        .unwrap();

        let owned_page_table = unsafe {
            OffsetPageTable::new(
                page_table_raw,
                VirtAddr::new_truncate(crate::memory::PHYSICAL_OFFSET as u64),
            )
        };

        let mut child = Self {
            pid: ProcessId::new(),
            name: self.name.clone(),
            parent: self.pid,
            status: ProgramStatus::Created,
            ticks_passed: 0,
            stack_frame: new_stack_frame,
            regs: self.regs.clone(),
            page_table_addr: (self.page_table_addr.0, Cr3::read().1),
            page_table: Some(owned_page_table),
            children: Vec::new(),
            proc_data: self.proc_data.clone(),
        };

        self.add_child(child.pid);

        self.regs.rax = u16::from(child.pid) as usize;
        child.regs.rax = 0;

        trace!(
            "Thread {}#{} forked to {}#{}.",
            self.name,
            self.pid,
            child.name,
            child.pid
        );
        trace!("{:#?}", &child);

        return child;
    }

    pub fn set_parent(&mut self, pid: ProcessId) {
        self.parent = pid;
    }

    pub fn children(&self) -> Vec<ProcessId> {
        self.children.clone()
    }

    pub fn not_drop_page_table(&mut self) {
        unsafe {
            self.page_table_addr.0 = PhysFrame::from_start_address_unchecked(PhysAddr::new(0))
        }
    }

    pub fn init_elf(&mut self, elf: &ElfFile) {
        let alloc = &mut *get_frame_alloc_for_sure();

        let mut page_table = self.page_table.take().unwrap();

        let code_segements = elf::load_elf(elf, &mut page_table, alloc).unwrap();
        elf::map_stack(STACK_BOT, STACK_PAGES, &mut page_table, alloc).unwrap();

        self.page_table = Some(page_table);
        self.proc_data.code_segements = Some(code_segements);
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let page_table = self.page_table.as_mut().unwrap();
        let frame_deallocator = &mut *get_frame_alloc_for_sure();
        let start_count = frame_deallocator.recycled_count();

        trace!("Free stack for {}#{}", self.name, self.pid);
        // only free stack, 0 is set by manager
        let stack_start = self.stack_frame.stack_pointer.as_u64() & STACK_START_MASK;
        elf::unmap_stack(
            stack_start,
            STACK_PAGES,
            page_table,
            frame_deallocator,
            true,
        )
        .unwrap();

        if self.page_table_addr.0.start_address().as_u64() != 0 {
            trace!("Clean up page_table for {}#{}", self.name, self.pid);
            unsafe {
                if let Some(ref mut segements) = self.proc_data.code_segements {
                    for range in segements {
                        for page in range {
                            if let Ok(ret) = page_table.unmap(page) {
                                frame_deallocator.deallocate_frame(ret.0);
                                ret.1.flush();
                            }
                        }
                    }
                }
                // free P1-P3
                page_table.clean_up(frame_deallocator);
                // free P4
                frame_deallocator.deallocate_frame(self.page_table_addr.0);
            }
        }

        let end_count = frame_deallocator.recycled_count();

        debug!(
            "Recycled {}({:.3}MiB) frames, {}({:.3}MiB) frames in total.",
            end_count - start_count,
            ((end_count - start_count) * 4) as f32 / 1024.0,
            end_count,
            (end_count * 4) as f32 / 1024.0
        );
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);
        f.field("name", &self.name);
        f.field("parent", &self.parent);
        f.field("status", &self.status);
        f.field("ticks_passed", &self.ticks_passed);
        f.field("children", &self.children);
        f.field("page_table_addr", &self.page_table_addr);
        f.field("status", &self.status);
        f.field("stack_top", &self.stack_frame.stack_pointer);
        f.field("cpu_flags", &self.stack_frame.cpu_flags);
        f.field("instruction_pointer", &self.stack_frame.instruction_pointer);
        f.field("regs", &self.regs);
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            " #{:-3} | #{:-3} | {:13} | {:8} | {:?}",
            u16::from(self.pid),
            u16::from(self.parent),
            self.name,
            self.ticks_passed,
            self.status
        )?;
        Ok(())
    }
}
