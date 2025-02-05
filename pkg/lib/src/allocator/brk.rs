use core::alloc::{GlobalAlloc, Layout};

use linked_list_allocator::LockedHeap;

use crate::*;

const INIT_HEAP_SIZE: usize = 2 * 1024 - 8; // 2 KiB
const MAX_HEAP_SIZE: usize = 8 * 1024 * 1024 - 8; // 8 MiB

#[global_allocator]
static ALLOCATOR: BrkAllocator = BrkAllocator::empty();

struct BrkAllocator {
    allocator: LockedHeap,
}

impl BrkAllocator {
    pub const fn empty() -> Self {
        Self {
            allocator: LockedHeap::empty(),
        }
    }

    pub unsafe fn init(&self) {
        let heap_start = sys_brk(None).unwrap();
        let heap_end = heap_start + INIT_HEAP_SIZE;

        let ret = sys_brk(Some(heap_end)).expect("Failed to allocate heap");

        assert!(ret == heap_end, "Failed to allocate heap");

        self.allocator
            .lock()
            .init(heap_start as *mut u8, INIT_HEAP_SIZE);
    }

    pub unsafe fn extend(&self) -> bool {
        let heap_size = self.allocator.lock().size();

        if heap_size > MAX_HEAP_SIZE {
            return false;
        }

        let extend_size = heap_size + 8;

        let heap_end = sys_brk(None).unwrap();
        let new_heap_end = heap_end + extend_size;
        let ret = sys_brk(Some(new_heap_end)).expect("Failed to allocate heap");

        assert!(ret == new_heap_end, "Failed to allocate heap");

        self.allocator.lock().extend(extend_size);

        true
    }
}

unsafe impl GlobalAlloc for BrkAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut ptr = self.allocator.alloc(layout);
        while ptr.is_null() && self.extend() {
            ptr = self.allocator.alloc(layout);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout)
    }
}

pub fn init() {
    unsafe { ALLOCATOR.init() };
}

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
