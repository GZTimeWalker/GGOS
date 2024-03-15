use alloc::borrow::ToOwned;
use core::ops::Deref;

/// A block of data.
#[derive(Clone)]
pub struct Block<const SIZE: usize> {
    contents: [u8; SIZE],
}

pub type Block512 = Block<512>;
pub type Block4096 = Block<4096>;

impl<const SIZE: usize> Block<SIZE> {
    /// Create a new block full of zeros.
    pub fn new(data: &[u8; SIZE]) -> Self {
        Self {
            contents: data.to_owned(),
        }
    }

    pub const fn size() -> usize {
        SIZE
    }

    pub fn as_u8_slice(&self) -> &[u8; SIZE] {
        &self.contents
    }

    pub fn as_mut_u8_slice(&mut self) -> &mut [u8; SIZE] {
        &mut self.contents
    }
}

impl<const SIZE: usize> Deref for Block<SIZE> {
    type Target = [u8; SIZE];

    /// For `&block[x..y] -> &[u8]`
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<const SIZE: usize> AsRef<[u8]> for Block<SIZE> {
    fn as_ref(&self) -> &[u8] {
        &self.contents
    }
}

impl<const SIZE: usize> AsMut<[u8]> for Block<SIZE> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.contents
    }
}

impl<const SIZE: usize> Default for Block<SIZE> {
    fn default() -> Self {
        Self {
            contents: [0u8; SIZE],
        }
    }
}

impl<const SIZE: usize> core::fmt::Debug for Block<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "Block:")?;
        for i in 0..16 {
            writeln!(
                f,
                "    {:016x} {:016x} {:016x} {:016x}",
                u64::from_be_bytes(self.contents[i * 32..i * 32 + 8].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 8..i * 32 + 16].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 16..i * 32 + 24].try_into().unwrap()),
                u64::from_be_bytes(self.contents[i * 32 + 24..i * 32 + 32].try_into().unwrap()),
            )?;
        }
        Ok(())
    }
}
