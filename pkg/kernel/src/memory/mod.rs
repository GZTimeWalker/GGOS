pub mod address;
pub mod allocator;
mod frames;

pub mod gdt;
pub mod user;

pub use address::*;
pub use frames::*;

pub fn init(boot_info: &'static boot::BootInfo) {
    let memory_map = &boot_info.memory_map;

    let mut mem_size = 0;
    let mut usable_mem_size = 0;

    for item in memory_map.iter() {
        mem_size += item.page_count;
        if item.ty == boot::MemoryType::CONVENTIONAL {
            usable_mem_size += item.page_count;
        }
    }

    let (size, unit) = crate::humanized_size(mem_size * PAGE_SIZE);
    info!("Physical Memory    : {:>7.*} {}", 3, size, unit);

    let (size, unit) = crate::humanized_size(usable_mem_size * PAGE_SIZE);
    info!("Free Usable Memory : {:>7.*} {}", 3, size, unit);

    let mut used = crate::proc::stack::KSTACK_DEF_PAGE as usize;

    for page in &boot_info.kernel_pages {
        used += page.count();
    }

    let (size, unit) = crate::humanized_size(used as u64 * PAGE_SIZE);
    info!("Kernel Used Memory : {:>7.*} {}", 3, size, unit);

    let size = used + usable_mem_size as usize;

    unsafe {
        init_FRAME_ALLOCATOR(BootInfoFrameAllocator::init(memory_map, used, size));
    }

    info!("Frame Allocator initialized.");
}
