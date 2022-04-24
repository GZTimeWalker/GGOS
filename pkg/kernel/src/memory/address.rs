pub const PHYSICAL_OFFSET: u64 = 0xFFFF800000000000;
pub const PAGE_SIZE: u64 = 4096;
pub const FRAME_SIZE: u64 = 4096;

pub fn physical_to_virtual(addr: u64) -> u64 {
    addr + PHYSICAL_OFFSET
}
