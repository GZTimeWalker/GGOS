//! Disk Device
//!
//! reference: <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs#L1350>

mod partition;

use crate::*;
pub use partition::*;

/// Identifies a Disk device
///
/// do not hold a reference to the disk device directly.
pub struct Disk<T>
where
    T: BlockDevice<Block512> + Clone,
{
    inner: T,
}

impl<T> Disk<T>
where
    T: BlockDevice<Block512> + Clone,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn volumes(&self) -> Result<[Volume<T>; 4]> {
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
#[derive(Clone, Copy)]
pub struct Volume<T>
where
    T: BlockDevice<Block512> + Clone,
{
    inner: T,
    pub meta: PartitionMetaData,
}

impl<T> Volume<T>
where
    T: BlockDevice<Block512> + Clone,
{
    pub fn new(inner: T, meta: PartitionMetaData) -> Self {
        Self { inner, meta }
    }
}

impl<T> core::fmt::Debug for Volume<T>
where
    T: BlockDevice<Block512> + Clone,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Volume").field("meta", &self.meta).finish()
    }
}

impl<T> BlockDevice<Block512> for Volume<T>
where
    T: BlockDevice<Block512> + Clone,
{
    fn block_count(&self) -> Result<usize> {
        self.inner.block_count()
    }

    fn read_block(&self, offset: usize, block: &mut Block512) -> Result<()> {
        let offset = offset + self.meta.begin_lba() as usize;
        self.inner.read_block(offset, block)
    }

    fn write_block(&self, offset: usize, block: &Block512) -> Result<()> {
        let offset = offset + self.meta.begin_lba() as usize;
        self.inner.write_block(offset, block)
    }
}
