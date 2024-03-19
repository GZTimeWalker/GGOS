//! MbrTable
//!
//! reference: <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs#L1350>
//! reference: <https://github.com/llenotre/maestro>

mod entry;

use core::marker::PhantomData;

use crate::*;
pub use entry::*;

pub struct MbrTable<T, B>
where
    T: BlockDevice<B> + Clone,
    B: BlockTrait,
{
    inner: T,
    partitions: [MbrPartition; 4],
    _block: PhantomData<B>,
}

impl<T, B> PartitionTable<T, B> for MbrTable<T, B>
where
    T: BlockDevice<B> + Clone,
    B: BlockTrait,
{
    fn parse(inner: T) -> Result<Self> {
        let mut block = B::default();
        inner.read_block(0, &mut block)?;

        let mut partitions = Vec::with_capacity(4);
        let buffer = block.as_ref();

        for i in 0..4 {
            partitions.push(MbrPartition::parse(
                buffer[0x1be + (i * 16)..0x1be + (i * 16) + 16]
                    .try_into()
                    .unwrap(),
            ));

            if partitions[i].is_active() {
                trace!("Partition {}: {:#?}", i, partitions[i]);
            }
        }

        Ok(Self {
            inner,
            partitions: partitions.try_into().unwrap(),
            _block: PhantomData,
        })
    }

    fn partitions(&self) -> Result<Vec<Partition<T, B>>> {
        let mut parts = Vec::new();

        for part in self.partitions {
            if part.is_active() {
                parts.push(Partition::new(
                    self.inner.clone(),
                    part.begin_lba() as usize,
                    part.total_lba() as usize,
                ));
            }
        }

        Ok(parts)
    }
}
