mod address;
mod frames;
mod paging;
pub mod allocator;

pub use frames::*;
pub use paging::*;
pub use address::*;

pub fn init(boot_info: &'static boot::BootInfo) {
    let physical_memory_offset = x86_64::VirtAddr::new_truncate(PHYSICAL_OFFSET as u64);
    let memory_map = &boot_info.memory_map;
    unsafe {
        init_PAGE_TABLE(paging::init(physical_memory_offset));
        init_FRAME_ALLOCATOR(BootInfoFrameAllocator::init(memory_map));
    }
}
