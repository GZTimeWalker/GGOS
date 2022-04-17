mod random;
mod disk;

pub use random::Random;
pub use disk::Disk;

#[derive(Debug, Eq, PartialEq)]
pub enum BlockError {
    Busy,
    UnknownDevice,
    Unknown,
    InvalidOperation,
    WithStatus(usize),
}

pub trait Device {
    fn read(&mut self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize, BlockError>;
    fn write(&mut self, buf: &[u8], offset: usize, size: usize) -> Result<usize, BlockError>;
}

pub trait BlockDevice: Device {
    fn block_size(&self) -> Result<usize, BlockError>;
}

pub trait FatDevice: BlockDevice {
    fn fat_meta(&self) -> &crate::FAT16Bpb;
    fn fat_table(&self) -> crate::FAT16Table;
}
