mod address;
mod frames;
mod paging;
pub mod allocator;

pub mod gdt;
pub mod user;
pub use frames::*;
pub use paging::*;
pub use address::*;

pub fn init(boot_info: &'static boot::BootInfo) {
    let physical_memory_offset = x86_64::VirtAddr::new_truncate(PHYSICAL_OFFSET as u64);
    let memory_map = &boot_info.memory_map;

    let mut mem_size = 0;

    for item in memory_map.iter() {
        mem_size += item.page_count * PAGE_SIZE;
    }

    let (size, unit) = humanized_size(mem_size);
    info!("Physical Memory Size: {:.3} {}", size, unit);

    let mut used = crate::process::KSTACK_DEF_PAGE as usize;

    for page in &boot_info.kernel_pages {
        used += page.count();
    }

    let (size, unit) = humanized_size(used as u64 * PAGE_SIZE);
    info!("Kernel Used Memory: {:.3} {}", size, unit);

    unsafe {
        init_PAGE_TABLE(paging::init(physical_memory_offset));
        init_FRAME_ALLOCATOR(BootInfoFrameAllocator::init(memory_map, used));
    }
}

fn humanized_size(size: u64) -> (f64, &'static str) {
    let bytes = size as f64;
    if bytes < 1024f64 {
        (bytes, "B")
    } else if (bytes / (1 << 10) as f64) < 1024f64 {
        (bytes / (1 << 10) as f64, "KiB")
    } else if (bytes / (1 << 20) as f64) < 1024f64 {
        (bytes / (1 << 20) as f64, "MiB")
    } else {
        (bytes / (1 << 30) as f64, "GiB")
    }
}
