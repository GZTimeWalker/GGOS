pub mod bpb;
pub mod directory;
pub mod direntry;
pub mod file;
pub mod impls;

use crate::*;
use directory::Directory;
use direntry::*;
use file::File;

use bpb::Fat16Bpb;

const BLOCK_SIZE: usize = 512;

/// Identifies a Fat16 Volume on the disk.
pub struct Fat16<T>
where
    T: BlockDevice<Block512>,
{
    pub(crate) handle: Fat16Handle<T>,
}

impl<T> Fat16<T>
where
    T: BlockDevice<Block512>,
{
    pub fn new(volume: T) -> Self {
        let handle = Arc::new(Fat16Impl::new(volume));
        Self { handle }
    }
}

type Fat16Handle<T> = Arc<Fat16Impl<T>>;

pub struct Fat16Impl<T>
where
    T: BlockDevice<Block512>,
{
    pub(crate) volume: T,
    pub bpb: Fat16Bpb,
    pub fat_start: usize,
    pub first_data_sector: usize,
    pub first_root_dir_sector: usize,
}

impl<T> core::fmt::Debug for Fat16<T>
where
    T: BlockDevice<Block512>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fat16")
            .field("bpb", &self.handle.bpb)
            .finish()
    }
}

impl<T> core::fmt::Debug for Fat16Impl<T>
where
    T: BlockDevice<Block512>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fat16Impl").field("bpb", &self.bpb).finish()
    }
}
