use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::TranslateResult::*;
use x86_64::structures::paging::*;

use crate::proc::PageTableContext;

pub const PAGE_SIZE: u64 = 4096;
pub const FRAME_SIZE: u64 = PAGE_SIZE;

pub static PHYSICAL_OFFSET: spin::Once<u64> = spin::Once::new();

pub fn init(boot_info: &'static boot::BootInfo) {
    PHYSICAL_OFFSET.call_once(|| boot_info.physical_memory_offset);

    info!("Physical Offset  : {:#x}", PHYSICAL_OFFSET.get().unwrap());
}

#[inline(always)]
pub fn physical_to_virtual(addr: u64) -> u64 {
    addr + PHYSICAL_OFFSET
        .get()
        .expect("PHYSICAL_OFFSET not initialized")
}

pub fn is_user_accessable(addr: usize) -> bool {
    let mapper = &mut PageTableContext::new().mapper();
    match mapper.translate(VirtAddr::new_truncate(addr as u64)) {
        Mapped {
            frame: _,
            offset: _,
            flags,
        } => flags.contains(PageTableFlags::USER_ACCESSIBLE),
        _ => false,
    }
}

pub fn as_user_str(ptr: usize, len: usize) -> Option<&'static str> {
    match core::str::from_utf8(as_user_slice(ptr, len)?) {
        Ok(s) => Some(s),
        Err(_) => {
            warn!("syscall: invalid utf8 string");
            None
        }
    }
}

pub fn as_user_slice<'a>(ptr: usize, len: usize) -> Option<&'a [u8]> {
    if !is_user_accessable(ptr) {
        warn!("syscall: invalid access to {:#x}", ptr);
        return None;
    }

    unsafe { Some(core::slice::from_raw_parts(ptr as *const u8, len)) }
}

pub fn as_user_slice_mut<'a>(ptr: usize, len: usize) -> Option<&'a mut [u8]> {
    if !is_user_accessable(ptr) {
        warn!("syscall: invalid access to {:#x}", ptr);
        return None;
    }

    unsafe { Some(core::slice::from_raw_parts_mut(ptr as *mut u8, len)) }
}
