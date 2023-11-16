//! Disk Device
//!
//! reference: <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs#L1350>

use super::*;
use crate::partition::PartitionMetaData;
use crate::*;

/// Identifies a Disk device
///
/// do not hold a reference to the disk device directly.
pub struct Disk<T>
where
    T: BlockDevice + Clone,
{
    inner: T,
}

impl<T> Disk<T>
where
    T: BlockDevice + Clone,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn volumes(&self) -> Result<[Volume<T>; 4], DeviceError> {
        let mut mbr = Block::default();
        self.inner.read_block(0, &mut mbr)?;
        let volumes = MBRPartitions::parse(mbr.as_u8_slice());

        Ok([
            Volume::new(self.inner.clone(), volumes.partitions[0]),
            Volume::new(self.inner.clone(), volumes.partitions[1]),
            Volume::new(self.inner.clone(), volumes.partitions[2]),
            Volume::new(self.inner.clone(), volumes.partitions[3]),
        ])
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

impl<T> BlockDevice for Volume<T>
where
    T: BlockDevice,
{
    fn block_count(&self) -> Result<usize, DeviceError> {
        self.inner.block_count()
    }

    fn read_block(&self, offset: usize, block: &mut Block) -> Result<(), DeviceError> {
        // trace!(
        //     "Read block offset: {}, Volume LBA start: {}",
        //     offset,
        //     self.meta.begin_lba()
        // );

        self.inner
            .read_block(offset + self.meta.begin_lba() as usize, block)
    }

    fn write_block(&self, offset: usize, block: &Block) -> Result<(), DeviceError> {
        // trace!(
        //     "Write block offset: {}, Volume LBA start: {}",
        //     offset,
        //     self.meta.begin_lba()
        // );

        self.inner
            .write_block(offset + self.meta.begin_lba() as usize, block)
    }
}
