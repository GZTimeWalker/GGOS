#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

use core::intrinsics::{copy_nonoverlapping, write_bytes};

use alloc::vec::Vec;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{mapper::*, *};
use x86_64::{align_up, PhysAddr, VirtAddr};
use xmas_elf::{program, ElfFile};

/// Map physical memory [0, max_addr)
///
/// to virtual space [offset, offset + max_addr)
pub fn map_physical_memory(
    offset: u64,
    max_addr: u64,
    page_table: &mut impl Mapper<Size2MiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    trace!("Mapping physical memory...");
    let start_frame = PhysFrame::containing_address(PhysAddr::new(0));
    let end_frame = PhysFrame::containing_address(PhysAddr::new(max_addr));

    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64() + offset));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map physical memory")
                .flush();
        }
    }
}

/// Map ELF file
///
/// for each segment, map current frame and set page table
pub fn map_elf(
    elf: &ElfFile,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    trace!("Mapping ELF file...{:?}", elf.input.as_ptr());
    let start = PhysAddr::new(elf.input.as_ptr() as u64);
    for segment in elf.program_iter() {
        map_segment(&segment, start, page_table, frame_allocator)?;
    }
    Ok(())
}

/// Unmap ELF file
pub fn unmap_elf(elf: &ElfFile, page_table: &mut impl Mapper<Size4KiB>) -> Result<(), UnmapError> {
    trace!("Unmapping ELF file...");
    let kernel_start = PhysAddr::new(elf.input.as_ptr() as u64);
    for segment in elf.program_iter() {
        unmap_segment(&segment, kernel_start, page_table)?;
    }
    Ok(())
}

/// 加载 ELF 文件栈
pub fn map_stack(
    addr: u64,
    pages: u64,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    trace!("Mapping stack at {:#x}", addr);
    // create a stack
    let stack_start = Page::containing_address(VirtAddr::new(addr));
    let stack_end = stack_start + pages;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in Page::range(stack_start, stack_end) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
    }

    trace!(
        "Stack hint: {:#x} -> {:#x}",
        addr,
        page_table
            .translate_page(stack_start)
            .unwrap()
            .start_address()
    );

    Ok(())
}

/// 卸载 ELF 文件栈
pub fn unmap_stack(
    addr: u64,
    pages: u64,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_deallocator: &mut impl FrameDeallocator<Size4KiB>,
    do_dealloc: bool,
) -> Result<(), UnmapError> {
    trace!("Unmapping stack at {:#x}", addr);

    let stack_start = Page::containing_address(VirtAddr::new(addr));

    trace!(
        "Stack hint: {:#x} -> {:#x}",
        addr,
        page_table
            .translate_page(stack_start)
            .unwrap()
            .start_address()
    );

    let stack_end = stack_start + pages;

    for page in Page::range(stack_start, stack_end) {
        let info = page_table.unmap(page)?;
        if do_dealloc {
            unsafe {
                frame_deallocator.deallocate_frame(info.0);
            }
        }
        info.1.flush();
    }

    Ok(())
}

fn map_segment(
    segment: &program::ProgramHeader,
    start: PhysAddr,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    if segment.get_type().unwrap() != program::Type::Load {
        return Ok(());
    }

    trace!("Mapping segment: {:#x?}", segment);
    let mem_size = segment.mem_size();
    let file_size = segment.file_size();
    let file_offset = segment.offset() & !0xfff;
    let phys_start_addr = start + file_offset;
    let virt_start_addr = VirtAddr::new(segment.virtual_addr());

    let start_page: Page = Page::containing_address(virt_start_addr);
    let start_frame = PhysFrame::containing_address(phys_start_addr);
    let end_frame = PhysFrame::containing_address(phys_start_addr + file_size - 1u64);

    let flags = segment.flags();
    let mut page_table_flags = PageTableFlags::PRESENT;

    if !flags.is_execute() {
        page_table_flags |= PageTableFlags::NO_EXECUTE;
    }

    if flags.is_write() {
        page_table_flags |= PageTableFlags::WRITABLE;
    }

    trace!("Segment page table flag: {:?}", page_table_flags);
    // DONT MAP ADDR DIRECTLY, ALLOCATE THEN COPY DATA
    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        let offset = frame - start_frame;
        let page = start_page + offset;
        unsafe {
            page_table
                .map_to(page, frame, page_table_flags, frame_allocator)?
                .flush();
        }
    }

    if mem_size > file_size {
        // .bss section (or similar), which needs to be zeroed
        let zero_start = virt_start_addr + file_size;
        let zero_end = virt_start_addr + mem_size;
        if zero_start.as_u64() & 0xfff != 0 {
            // A part of the last mapped frame needs to be zeroed. This is
            // not possible since it could already contains parts of the next
            // segment. Thus, we need to copy it before zeroing.

            let new_frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            type PageArray = [u64; Size4KiB::SIZE as usize / 8];

            let last_page = Page::containing_address(virt_start_addr + file_size - 1u64);
            let last_page_ptr = end_frame.start_address().as_u64() as *mut PageArray;
            let temp_page_ptr = new_frame.start_address().as_u64() as *mut PageArray;

            unsafe {
                // copy contents
                temp_page_ptr.write(last_page_ptr.read());
            }

            // remap last page
            if let Err(e) = page_table.unmap(last_page) {
                return Err(match e {
                    UnmapError::ParentEntryHugePage => MapToError::ParentEntryHugePage,
                    UnmapError::PageNotMapped => unreachable!(),
                    UnmapError::InvalidFrameAddress(_) => unreachable!(),
                });
            }

            unsafe {
                page_table
                    .map_to(last_page, new_frame, page_table_flags, frame_allocator)?
                    .flush();
            }
        }

        // Map additional frames.
        let start_page: Page =
            Page::containing_address(VirtAddr::new(align_up(zero_start.as_u64(), Size4KiB::SIZE)));
        let end_page = Page::containing_address(zero_end);
        for page in Page::range_inclusive(start_page, end_page) {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            unsafe {
                page_table
                    .map_to(page, frame, page_table_flags, frame_allocator)?
                    .flush();
            }
        }

        // zero bss
        unsafe {
            core::ptr::write_bytes(
                zero_start.as_mut_ptr::<u8>(),
                0,
                (mem_size - file_size) as usize,
            );
        }
    }
    Ok(())
}

/// Load & Map ELF file
///
/// for each segment, load code to new frame and set page table
pub fn load_elf(
    elf: &ElfFile,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<Vec<PageRangeInclusive>, MapToError<Size4KiB>> {
    trace!("Loading ELF file...{:?}", elf.input.as_ptr());
    elf.program_iter()
        .filter(|segment| segment.get_type().unwrap() == program::Type::Load)
        .map(|segment| load_segment(elf, &segment, page_table, frame_allocator))
        .collect()
}

// load segments to new allocated frames
fn load_segment(
    elf: &ElfFile,
    segment: &program::ProgramHeader,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<PageRangeInclusive, MapToError<Size4KiB>> {
    trace!("Loading & mapping segment: {:#x?}", segment);
    let mem_size = segment.mem_size();
    let file_size = segment.file_size();
    let file_offset = segment.offset() & !0xfff;
    let virt_start_addr = VirtAddr::new(segment.virtual_addr());

    let flags = segment.flags();
    let mut page_table_flags = PageTableFlags::PRESENT;

    if !flags.is_execute() {
        page_table_flags |= PageTableFlags::NO_EXECUTE;
    }

    if flags.is_write() {
        page_table_flags |= PageTableFlags::WRITABLE;
    }

    trace!("Segment page table flag: {:?}", page_table_flags);

    let start_page = Page::containing_address(virt_start_addr);
    let end_page = Page::containing_address(virt_start_addr + file_size - 1u64);
    let pages = Page::range_inclusive(start_page, end_page);

    let data = unsafe { elf.input.as_ptr().add(file_offset as usize) };

    for (idx, page) in pages.enumerate() {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let offset = idx as u64 * page.size();
        let count = if file_size - offset < page.size() {
            file_size - offset
        } else {
            page.size()
        };

        trace!(
            "Map page: {:#x} -> {:#x} ({}/{})",
            page.start_address().as_u64(),
            frame.start_address().as_u64(),
            offset,
            file_size
        );

        unsafe {
            trace!(
                "Copying data: {:#x} -> {:#x}",
                data as u64 + idx as u64 * page.size(),
                frame.start_address().as_u64()
            );

            copy_nonoverlapping(
                data.add(idx * page.size() as usize),
                frame.start_address().as_u64() as *mut u8,
                count as usize,
            );

            page_table
                .map_to(page, frame, page_table_flags, frame_allocator)?
                .flush();

            if count < page.size() {
                // zero the rest of the page
                trace!(
                    "Zeroing rest of the page: {:#x}",
                    page.start_address().as_u64()
                );
                write_bytes(
                    (frame.start_address().as_u64() + count) as *mut u8,
                    0,
                    (page.size() - count) as usize,
                );
            }
        }
    }

    if mem_size > file_size {
        // .bss section (or similar), which needs to be zeroed
        let zero_start = virt_start_addr + file_size;
        let zero_end = virt_start_addr + mem_size;

        // Map additional frames.
        let start_page: Page =
            Page::containing_address(VirtAddr::new(align_up(zero_start.as_u64(), Size4KiB::SIZE)));
        let end_page = Page::containing_address(zero_end);

        for page in Page::range_inclusive(start_page, end_page) {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            unsafe {
                page_table
                    .map_to(page, frame, page_table_flags, frame_allocator)?
                    .flush();
            // zero bss

                write_bytes(
                    frame.start_address().as_u64() as *mut u8,
                    0,
                    page.size() as usize,
                );
            }
        }

    }

    let end_page = Page::containing_address(virt_start_addr + mem_size - 1u64);
    Ok(Page::range_inclusive(start_page, end_page))
}

fn unmap_segment(
    segment: &program::ProgramHeader,
    kernel_start: PhysAddr,
    page_table: &mut impl Mapper<Size4KiB>,
) -> Result<(), UnmapError> {
    if segment.get_type().unwrap() != program::Type::Load {
        return Ok(());
    }
    trace!("Unmapping segment: {:#x?}", segment);
    let mem_size = segment.mem_size();
    let file_size = segment.file_size();
    let file_offset = segment.offset() & !0xfff;
    let phys_start_addr = kernel_start + file_offset;
    let virt_start_addr = VirtAddr::new(segment.virtual_addr());

    let start_page: Page = Page::containing_address(virt_start_addr);
    let start_frame = PhysFrame::<Size4KiB>::containing_address(phys_start_addr);
    let end_frame = PhysFrame::containing_address(phys_start_addr + file_size - 1u64);

    let flags = segment.flags();
    let mut page_table_flags = PageTableFlags::PRESENT;
    if !flags.is_execute() {
        page_table_flags |= PageTableFlags::NO_EXECUTE
    };
    if flags.is_write() {
        page_table_flags |= PageTableFlags::WRITABLE
    };

    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        let offset = frame - start_frame;
        let page = start_page + offset;
        page_table.unmap(page)?.1.flush();
    }

    if mem_size > file_size {
        // .bss section (or similar), which needs to be zeroed
        let zero_start = virt_start_addr + file_size;
        let zero_end = virt_start_addr + mem_size;
        if zero_start.as_u64() & 0xfff != 0 {
            // A part of the last mapped frame needs to be zeroed. This is
            // not possible since it could already contains parts of the next
            // segment. Thus, we need to copy it before zeroing.

            let last_page = Page::containing_address(virt_start_addr + file_size - 1u64);

            page_table.unmap(last_page)?.1.flush();
        }

        // Map additional frames.
        let start_page: Page =
            Page::containing_address(VirtAddr::new(align_up(zero_start.as_u64(), Size4KiB::SIZE)));
        let end_page = Page::containing_address(zero_end);
        for page in Page::range_inclusive(start_page, end_page) {
            page_table.unmap(page)?.1.flush();
        }
    }
    Ok(())
}
