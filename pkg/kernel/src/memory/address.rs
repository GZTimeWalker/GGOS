pub const PHYSICAL_OFFSET: usize = 0xFFFF800000000000;

pub fn physical_to_virtual(addr: usize) -> usize {
    addr + PHYSICAL_OFFSET
}
