mod random;
pub mod disk;

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
    fn read(&self, buf: &mut [T], offset: usize, size: usize) -> Result<usize, DeviceError>;

    // TODO: implement write
    //fn write(&mut self, buf: &[T], offset: usize, size: usize) -> Result<usize, DeviceError>;
}

pub trait BlockDevice: Device<Block> {
    fn block_count(&self) -> Result<usize, DeviceError>;
    fn read_block(&self, offset: usize) -> Result<Block, DeviceError>;

    // TODO: implement write
}
