mod random;

pub mod disk;
pub mod fat16;

pub use disk::*;
pub use fat16::*;
pub use random::Random;

use crate::dir_entry::FilenameError;

pub use super::block::Block;

#[derive(Debug, Eq, PartialEq)]
pub enum DeviceError {
    Busy,
    UnknownDevice,
    Unknown,
    InvalidOperation,
    ReadError,
    WriteError,
    WithStatus(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub enum VolumeError {
    NotInSector,
    FileNotFound,
    InvalidOperation,
    BadCluster,
    EndOfFile,
    NotADirectory,
    NotAFile,
    ReadOnly,
    Unsupported,
    BufferTooSmall,
    DeviceError(DeviceError),
    FileNameError(FilenameError),
}

pub trait Device<T> {
    fn read(&self, buf: &mut [T], offset: usize, size: usize) -> Result<usize, DeviceError>;
    fn write(&mut self, buf: &[T], offset: usize, size: usize) -> Result<usize, DeviceError>;
}

pub trait BlockDevice: Send + Sync {
    fn block_count(&self) -> Result<usize, DeviceError>;
    fn read_block(&self, offset: usize, block: &mut Block) -> Result<(), DeviceError>;
    fn write_block(&self, offset: usize, block: &Block) -> Result<(), DeviceError>;
}
