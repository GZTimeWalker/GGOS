pub const BLOCK_SIZE: usize = 512;

#[derive(Clone)]
pub struct Block {
    pub addr: usize,
    pub buf: [u8; BLOCK_SIZE],
}
