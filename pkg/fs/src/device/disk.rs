//! Disk Device
//!
//! reference: https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs#L1350

use super::*;
use crate::*;
use crate::partition::PartitionMetaData;

/// Identifies a Disk device
///
/// do not hold a reference to the disk device directly.
pub struct Disk<T>
where
    T: BlockDevice + Clone,
{
    inner: T
}

impl<T> Disk<T>
where
    T: BlockDevice + Clone,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn volumes(&mut self) -> [Volume<T>; 4] {
        let mbr = self.inner.read_block(0).unwrap();
        let volumes = MBRPartitions::parse(mbr.inner());
        [
            Volume::new(self.inner.clone(), volumes.partitions[0]),
            Volume::new(self.inner.clone(), volumes.partitions[1]),
            Volume::new(self.inner.clone(), volumes.partitions[2]),
            Volume::new(self.inner.clone(), volumes.partitions[3]),
        ]
    }
}

/// Identifies a Volume on the disk.
pub struct Volume<T>
where
    T: BlockDevice,
{
    inner: T,
    pub meta: PartitionMetaData,
}

impl<T> Volume<T>
where
    T: BlockDevice,
{
    pub fn new(inner: T, meta: PartitionMetaData) -> Self {
        Self { inner, meta }
    }
}

impl<T> Device<Block> for Volume<T>
where
    T: BlockDevice,
{
    fn read(&self, buf: &mut [Block], offset: usize, size: usize) -> Result<usize, DeviceError> {
        self.inner.read(buf, offset + self.meta.begin_lba() as usize, size)
    }

    fn write(&mut self, buf: &[Block], offset: usize, size: usize) -> Result<usize, DeviceError> {
        self.inner.write(buf, offset + self.meta.begin_lba() as usize, size)
    }
}

impl<T> BlockDevice for Volume<T>
where
    T: BlockDevice,
{
    fn block_count(&self) -> Result<usize, DeviceError> {
        self.inner.block_count()
    }

    fn read_block(&self, offset: usize) -> Result<Block, DeviceError> {
        trace!(
            "read_block offset: {}, volume lba start: {}",
            offset,
            self.meta.begin_lba()
        );

        let block = self
            .inner
            .read_block(offset + self.meta.begin_lba() as usize);

        if let Ok(block_value) = block {
            // trace!("{:?}", block_value);
            return Ok(block_value);
        }

        block
    }

    fn write_block(&mut self, offset: usize, block: &Block) -> Result<(), DeviceError> {
        trace!(
            "write_block offset: {}, volume lba start: {}",
            offset,
            self.meta.begin_lba()
        );

        self.inner.write_block(offset + self.meta.begin_lba() as usize, block)
    }
}
