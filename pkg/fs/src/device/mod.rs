mod random;

pub use random::Random;

use super::block::Block;

#[derive(Debug, Eq, PartialEq)]
pub enum DeviceError {
    Busy,
    UnknownDevice,
    Unknown,
    InvalidOperation,
    WithStatus(usize),
}

pub trait Device<T> {
    fn read(&mut self, buf: &mut [T], offset: usize, size: usize) -> Result<usize, DeviceError>;
    fn write(&mut self, buf: &[T], offset: usize, size: usize) -> Result<usize, DeviceError>;
}

pub trait BlockDevice: Device<Block> {
    fn block_count(&self) -> usize;
}

pub trait FatDevice: BlockDevice {
    fn fat_meta(&self) -> &crate::FAT16Bpb;
    fn fat_table(&self) -> crate::FAT16Table;
}
