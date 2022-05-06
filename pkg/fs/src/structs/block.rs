use crate::alloc::borrow::ToOwned;

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

    pub fn inner_mut(&mut self) -> &mut [u8; Block::SIZE] {
        &mut self.contents
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            contents: [0u8; Block::SIZE],
        }
    }
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Block:\n")?;
        for i in 0..16 {
            write!(
                f,
                "    {:016x} {:016x} {:016x} {:016x}\n",
                u64::from_be_bytes(self.contents[i * 32..i * 32 + 8].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 8..i * 32 + 16].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 16..i * 32 + 24].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 24..i * 32 + 32].try_into().unwrap()),
            )?;
        }
        Ok(())
    }
}
