// reference: https://github.com/xfoxfu/rust-xos/blob/main/kernel/src/allocator.rs

use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0xFFFF_FF80_0000_0000;
pub const HEAP_SIZE: usize = 512 * 1024; // 512 KiB

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    init_heap().expect("Heap Initialization Failed.");
    super::user::init_user_heap().expect("User Heap Initialization Failed.");
    info!("Heap Initialized.");
}

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let mapper = &mut *super::get_page_table_for_sure();
    let frame_allocator = &mut *super::get_frame_alloc_for_sure();

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    debug!("Kernel Heap Start : {:?}", page_range.start);
    debug!("Kernel Heap End   : {:?}", page_range.end);

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
