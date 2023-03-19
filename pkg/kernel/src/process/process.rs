use super::ProcessId;
use super::*;
use crate::filesystem::StdIO;
use crate::memory::gdt::get_user_selector;
use crate::memory::{self, *};
use crate::utils::{Registers, RegistersValue, Resource};
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use boot::KernelPages;
use core::intrinsics::copy_nonoverlapping;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::{InterruptStackFrame, InterruptStackFrameValue};
use x86_64::structures::paging::mapper::{CleanUp, MapToError};
use x86_64::structures::paging::page::{PageRange, PageRangeInclusive};
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
    code_segments: Option<Vec<PageRangeInclusive>>,
    stack_segment: Option<PageRange>,
    file_handles: BTreeMap<u8, Resource>,
    pub code_memory_usage: usize,
    pub stack_memory_usage: usize,
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
        let code_segments = None;
        let stack_segment = None;
        Self {
            env,
            code_segments,
            stack_segment,
            file_handles,
            code_memory_usage: 0,
            stack_memory_usage: 0,
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

    pub fn set_stack(mut self, start: u64, size: u64) -> Self {
        let start = Page::containing_address(VirtAddr::new(start));
        self.stack_segment = Some(Page::range(start, start + size));
        self.stack_memory_usage = size as usize;
        self
    }

    pub fn set_kernel_code(mut self, pages: &KernelPages) -> Self {
        let mut size = 0;
        let owned_pages = pages
            .iter()
            .map(|page| {
                size += page.count();
                PageRangeInclusive::from(page.clone())
            })
            .collect();
        self.code_segments = Some(owned_pages);
        self.code_memory_usage = size;
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
        let selector = get_user_selector();
        self.stack_frame.code_segment = selector.user_code_selector.0 as u64;
        self.stack_frame.stack_segment = selector.user_data_selector.0 as u64;
        trace!("Init stack frame: {:#?}", &self.stack_frame);
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
                physical_to_virtual(page_table_source.start_address().as_u64()) as *mut PageTable,
                physical_to_virtual(page_table_addr.start_address().as_u64()) as *mut PageTable,
                1,
            );
        }

        // 3. create page table object
        let page_table = Self::page_table_from_phyframe(page_table_addr);

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

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        if let Some(stack_range) = self.proc_data.stack_segment {
            let addr = addr.as_u64();
            let cur_stack_bot = stack_range.start.start_address().as_u64();
            trace!("Current stack bot: {:#x}", cur_stack_bot);
            trace!("Address to access: {:#x}", addr);
            addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
        } else {
            false
        }
    }

    pub fn try_alloc_new_stack_page(&mut self, addr: VirtAddr) -> Result<(), MapToError<Size4KiB>> {
        let alloc = &mut *get_frame_alloc_for_sure();
        let start_page = Page::<Size4KiB>::containing_address(addr);
        let pages = self.proc_data.stack_segment.unwrap().start - start_page;
        let page_table = self.page_table.as_mut().unwrap();
        trace!(
            "Fill missing pages...[{:#x} -> {:#x})",
            start_page.start_address().as_u64(),
            self.proc_data
                .stack_segment
                .unwrap()
                .start
                .start_address()
                .as_u64()
        );

        elf::map_range(addr.as_u64(), pages, page_table, alloc, true)?;

        let end_page = self.proc_data.stack_segment.unwrap().end;
        let new_stack = PageRange {
            start: start_page,
            end: end_page,
        };

        self.proc_data.stack_memory_usage = new_stack.count();
        self.proc_data.stack_segment = Some(new_stack);

        Ok(())
    }

    fn clone_range(&self, cur_addr: u64, dest_addr: u64, size: usize) {
        trace!("Clone range: {:#x} -> {:#x}", cur_addr, dest_addr);
        unsafe {
            copy_nonoverlapping::<u8>(
                cur_addr as *mut u8,
                dest_addr as *mut u8,
                size * Size4KiB::SIZE as usize,
            );
        }
    }

    fn page_table_from_phyframe(frame: PhysFrame) -> OffsetPageTable<'static> {
        unsafe {
            OffsetPageTable::new(
                (physical_to_virtual(frame.start_address().as_u64()) as *mut PageTable)
                    .as_mut()
                    .unwrap(),
                VirtAddr::new_truncate(crate::memory::PHYSICAL_OFFSET as u64),
            )
        }
    }

    pub fn fork(&mut self) -> Process {
        // deep clone page is not impl yet, so we put the thread stack to a new mem
        let frame_alloc = &mut *get_frame_alloc_for_sure();

        // use the same page table with the parent, but remap stack with new offset
        let stack_info = self.proc_data.stack_segment.unwrap();

        let mut new_stack_base = stack_info.start.start_address().as_u64()
            - (self.children.len() as u64 + 1) * STACK_MAX_SIZE;

        while elf::map_range(
            new_stack_base,
            stack_info.count() as u64,
            self.page_table.as_mut().unwrap(),
            frame_alloc,
            true,
        )
        .is_err()
        {
            trace!("Map thread stack to {:#x} failed.", new_stack_base);
            new_stack_base -= STACK_MAX_SIZE; // stack grow down
        }

        debug!("Map thread stack to {:#x} succeed.", new_stack_base);

        let cur_stack_base = stack_info.start.start_address().as_u64();
        // make new stack frame
        let mut new_stack_frame = self.stack_frame.clone();
        // cal new stack pointer
        new_stack_frame.stack_pointer += new_stack_base - cur_stack_base;
        // clone new stack content
        self.clone_range(cur_stack_base, new_stack_base, stack_info.count());

        // create owned page table (same as parent)
        let owned_page_table = Self::page_table_from_phyframe(self.page_table_addr.0);
        // clone proc data
        let mut owned_proc_data = self.proc_data.clone();
        // record new stack range
        let stack = Page::range(
            Page::containing_address(VirtAddr::new_truncate(new_stack_base)),
            Page::containing_address(VirtAddr::new_truncate(
                new_stack_base + stack_info.count() as u64 * Size4KiB::SIZE,
            )),
        );
        // use shared code segment, only record the new stack usage
        owned_proc_data.stack_memory_usage = stack.count();
        owned_proc_data.code_memory_usage = 0;
        owned_proc_data.stack_segment = Some(stack);

        // create new process
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
            proc_data: owned_proc_data,
        };

        // record child pid
        self.add_child(child.pid);

        // fork ret value
        self.regs.rax = u16::from(child.pid) as usize;
        child.regs.rax = 0;

        debug!(
            "Thread {}#{} forked to {}#{}.",
            self.name, self.pid, child.name, child.pid
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

    pub fn memory_usage(&self) -> usize {
        self.proc_data.code_memory_usage + self.proc_data.stack_memory_usage
    }

    pub fn not_drop_page_table(&mut self) {
        unsafe {
            self.page_table_addr.0 = PhysFrame::from_start_address_unchecked(PhysAddr::new(0))
        }
    }

    pub fn init_elf(&mut self, elf: &ElfFile) {
        let alloc = &mut *get_frame_alloc_for_sure();

        let mut page_table = self.page_table.take().unwrap();

        let code_segments =
            elf::load_elf(elf, PHYSICAL_OFFSET, &mut page_table, alloc, true).unwrap();

        let stack_segment =
            elf::map_range(STACT_INIT_BOT, STACK_DEF_PAGE, &mut page_table, alloc, true).unwrap();

        // record memory usage
        self.proc_data.code_memory_usage = code_segments
            .iter()
            .map(|seg| seg.count())
            .fold(0, |acc, x| acc + x);

        self.proc_data.stack_memory_usage = stack_segment.count();

        self.page_table = Some(page_table);
        self.proc_data.code_segments = Some(code_segments);
        self.proc_data.stack_segment = Some(stack_segment);
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let page_table = self.page_table.as_mut().unwrap();
        let frame_deallocator = &mut *get_frame_alloc_for_sure();
        let start_count = frame_deallocator.recycled_count();

        let stack = self.proc_data.stack_segment.unwrap();

        trace!(
            "Free stack for {}#{}: [{:#x} -> {:#x}) ({} frames)",
            self.name,
            self.pid,
            stack.start.start_address(),
            stack.end.start_address(),
            stack.count()
        );

        elf::unmap_range(
            stack.start.start_address().as_u64(),
            stack.count() as u64,
            page_table,
            frame_deallocator,
            true,
        )
        .unwrap();

        if self.page_table_addr.0.start_address().as_u64() != 0 {
            trace!("Clean up page_table for {}#{}", self.name, self.pid);
            unsafe {
                if let Some(ref mut segments) = self.proc_data.code_segments {
                    for range in segments {
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
        f.field("stack", &self.proc_data.stack_segment);
        f.field("regs", &self.regs);
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = memory::humanized_size(self.memory_usage() as u64 * 4096);
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:>5.1} {} | {:?}",
            u16::from(self.pid),
            u16::from(self.parent),
            self.name,
            self.ticks_passed,
            size,
            unit,
            self.status
        )?;
        Ok(())
    }
}
