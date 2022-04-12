use x86_64::VirtAddr;
use boot::MemoryMap;

mod address;
mod frames;
mod paging;
pub mod allocator;

pub use frames::*;
pub use paging::*;
pub use address::*;

pub unsafe fn init(physical_memory_offset: VirtAddr, memory_map: &'static MemoryMap) {
    init_PAGE_TABLE(paging::init(physical_memory_offset));
    init_FRAME_ALLOCATOR(BootInfoFrameAllocator::init(memory_map));
}
