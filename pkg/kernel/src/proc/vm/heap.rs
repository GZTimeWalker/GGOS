use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use x86_64::{
    VirtAddr,
    structures::paging::{Page, mapper::UnmapError},
};

use super::*;

// user process runtime heap
// 0x100000000 bytes -> 4GiB
// from 0x0000_2000_0000_0000 to 0x0000_2000_ffff_fff8
pub const HEAP_START: u64 = 0x2000_0000_0000;
pub const HEAP_PAGES: u64 = 0x100000;
pub const HEAP_SIZE: u64 = HEAP_PAGES * crate::memory::PAGE_SIZE;
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE - 8;

/// User process runtime heap
///
/// always page aligned, the range is [base, end)
pub struct Heap {
    base: VirtAddr,
    end: Arc<AtomicU64>,
}

impl Heap {
    pub fn fork(&self) -> Self {
        Self {
            base: self.base,
            end: self.end.clone(),
        }
    }

    pub fn brk(
        &self,
        new_end: Option<VirtAddr>,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Option<VirtAddr> {
        if new_end.is_none() {
            return Some(VirtAddr::new(self.end.load(Ordering::Relaxed)));
        }

        let new_end = new_end.unwrap();

        if new_end > self.base + HEAP_SIZE || new_end < self.base {
            error!("Heap brk: new_end is out of heap range");
            return None;
        }

        let cur_end = self.end.load(Ordering::Acquire);
        // heap: [base, cur_end, cur_end + 1) or [base, base)
        let mut cur_end_page = Page::containing_address(VirtAddr::new(cur_end));
        if cur_end != self.base.as_u64() {
            // the heap is already initialized, add 1 to exclude cur_end
            cur_end_page += 1;
        }
        // heap: [base, new_end, new_end + 1) or [base, base)
        let mut new_end_page = Page::containing_address(new_end);
        if new_end != self.base {
            // the new_end is not the base, add 1 to include new_end
            new_end_page += 1;
        }

        debug!("Heap end addr: {:#x} -> {:#x}", cur_end, new_end.as_u64());
        debug!(
            "Heap end page: {:#x} -> {:#x}",
            cur_end_page.start_address().as_u64(),
            new_end_page.start_address().as_u64()
        );

        match new_end_page.cmp(&cur_end_page) {
            core::cmp::Ordering::Greater => {
                // heap: [base, cur_end, new_end) -> map [cur_end, new_end - 1]
                let range = Page::range_inclusive(cur_end_page, new_end_page - 1);
                elf::map_range(range, mapper, alloc, true).ok()?;
            }
            core::cmp::Ordering::Less => {
                // heap: [base, new_end, cur_end) -> unmap [new_end, cur_end - 1]
                let range = Page::range_inclusive(new_end_page, cur_end_page - 1);
                elf::unmap_range(range, mapper, alloc, true).ok()?;
            }
            core::cmp::Ordering::Equal => {}
        }

        self.end.store(new_end.as_u64(), Ordering::Release);
        Some(new_end)
    }
}

impl VmPartExt for Heap {
    fn empty() -> Self {
        Self {
            base: VirtAddr::new(HEAP_START),
            end: Arc::new(AtomicU64::new(HEAP_START)),
        }
    }

    fn clean_up(
        &mut self,
        mapper: MapperRef,
        dealloc: FrameAllocatorRef,
    ) -> Result<(), UnmapError> {
        if self.memory_usage() == 0 {
            return Ok(());
        }

        // load the current end address and reset it to base
        let end_addr = self.end.swap(self.base.as_u64(), Ordering::Relaxed);

        let start_page = Page::containing_address(self.base);
        let end_page = Page::containing_address(VirtAddr::new(end_addr));
        let range = Page::range_inclusive(start_page, end_page);

        elf::unmap_range(range, mapper, dealloc, true)?;

        Ok(())
    }

    fn memory_usage(&self) -> u64 {
        self.end.load(Ordering::Relaxed) - self.base.as_u64()
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("base", &format_args!("{:#x}", self.base.as_u64()))
            .field(
                "end",
                &format_args!("{:#x}", self.end.load(Ordering::Relaxed)),
            )
            .finish()
    }
}
