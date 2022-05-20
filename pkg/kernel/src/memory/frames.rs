// reference: https://github.com/phil-opp/blog_os/blob/post-09/src/memory.rs
// reference: https://github.com/xfoxfu/rust-xos/blob/main/kernel/src/memory.rs

use alloc::vec::Vec;
use boot::{MemoryMap, MemoryType};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB, FrameDeallocator};
use x86_64::PhysAddr;

once_mutex!(pub FRAME_ALLOCATOR: BootInfoFrameAllocator);

guard_access_fn! {
    pub get_frame_alloc(FRAME_ALLOCATOR: BootInfoFrameAllocator)
}

type BootInfoFrameIter = impl Iterator<Item = PhysFrame>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    frames: BootInfoFrameIter,
    used: usize,
    recycled: Vec<PhysFrame>,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &MemoryMap) -> Self {
        BootInfoFrameAllocator {
            frames: create_frame_iter(memory_map),
            used: 0,
            recycled: Vec::new(),
        }
    }

    pub fn frames_used(&self) -> usize {
        self.used
    }

    pub fn recycled_count(&self) -> usize {
        self.recycled.len()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(frame) = self.recycled.pop() {
            Some(frame)
        } else {
            self.used += 1;
            self.frames.next()
        }
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.recycled.push(frame);
    }
}

unsafe fn create_frame_iter(memory_map: &MemoryMap) -> BootInfoFrameIter {
    memory_map
        .clone().into_iter()
        // get usable regions from memory map
        .filter(|r| r.ty == MemoryType::CONVENTIONAL)
        // align to page boundary
        .flat_map(|r| (0..r.page_count).map(move |v| (v * 4096 + r.phys_start)))
        // create `PhysFrame` types from the start addresses
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}
