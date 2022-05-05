#[derive(Clone)]
pub struct Block {
    pub contents: [u8; Block::SIZE],
}

impl Block {
    pub const SIZE: usize = 512;

    /// Create a new block full of zeros.
    pub fn new(data: &[u8; Block::SIZE]) -> Self {
        Self {
            contents: data.to_owned(),
        }
    }

    pub fn inner(&self) -> &[u8; Block::SIZE] {
        &self.contents
    }


}

impl Default for Block {
    fn default() -> Self {
        Self {
            contents: [0u8; Block::SIZE],
        }
    }
}
