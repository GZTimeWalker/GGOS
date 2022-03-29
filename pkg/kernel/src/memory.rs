pub const PHYSICAL_OFFSET: u64 = 0xFFFF800000000000;

pub fn physical_to_virtual(addr: usize) -> usize {
    addr + PHYSICAL_OFFSET as usize
}
