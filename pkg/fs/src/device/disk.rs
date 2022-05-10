//! Disk Device
//!
//! reference: https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs#L1350

use super::*;
use crate::Block;
use crate::{dir_entry::*, partition::PartitionMetaData, structs::*};

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
            trace!("{:?}", block_value);
            return Ok(block_value);
        }

        block
    }
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

/// Identifies a FAT16 Volume on the disk.
pub struct FAT16Volume<T>
where
    T: BlockDevice,
{
    volume: Volume<T>,
    pub bpb: FAT16Bpb,
    pub fat_start: usize,
    pub first_data_sector: usize,
    pub first_root_dir_sector: usize,
}

impl<T> Device<Block> for FAT16Volume<T>
where
    T: BlockDevice,
{
    fn read(&self, buf: &mut [Block], offset: usize, size: usize) -> Result<usize, DeviceError> {
        self.volume.read(buf, offset, size)
    }

    fn write(&mut self, buf: &[Block], offset: usize, size: usize) -> Result<usize, DeviceError> {
        self.volume.write(buf, offset, size)
    }
}

impl<T> BlockDevice for FAT16Volume<T>
where
    T: BlockDevice,
{
    fn block_count(&self) -> Result<usize, DeviceError> {
        self.volume.block_count()
    }

    fn read_block(&self, offset: usize) -> Result<Block, DeviceError> {
        self.volume.read_block(offset)
    }
}

impl<T> FAT16Volume<T>
where
    T: BlockDevice,
{
    pub fn new(volume: Volume<T>) -> Self {
        let block = volume.read_block(0).unwrap();
        let bpb = FAT16Bpb::new(block.inner()).unwrap();

        debug!("Loading FAT16 Volume: \n{:?}", bpb);

        // FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let root_dir_size =
            ((bpb.root_entries_count() as usize * DirEntry::LEN) + Block::SIZE - 1) / Block::SIZE;

        let fat_start = bpb.reserved_sector_count() as usize;
        let first_root_dir_sector =
            fat_start + (bpb.fat_count() as usize * bpb.sectors_per_fat() as usize);
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            volume,
            bpb,
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn iterate_dir<F>(&self, dir: &Directory, mut func: F) -> Result<(), VolumeError>
    where
        F: FnMut(&DirEntry),
    {
        debug!("Iterating directory: {:?}", dir);
        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                let block = self.volume.read_block(sector).unwrap();
                for entry in 0..Block::SIZE / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;
                    trace!("Entry: {}..{}", start, end);
                    let dir_entry = DirEntry::parse(&block.inner()[start..end]).map_err(|x| VolumeError::FileNameError(x))?;

                    if dir_entry.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        debug!("found file {}", dir_entry.filename());
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }

    /// Get an entry from the given directory
    pub fn find_directory_entry(
        &self,
        dir: &Directory,
        name: &str,
    ) -> Result<DirEntry, VolumeError> {
        let match_name = ShortFileName::parse(name)
            .map_err(|x| VolumeError::FileNameError(x))?;

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                match self.find_entry_in_sector(&match_name, sector) {
                    Err(VolumeError::NotInSector) => continue,
                    x => return x,
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Err(VolumeError::FileNotFound)
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                let first_sector_of_cluster = (c - 2) * self.bpb.sectors_per_cluster() as u32;
                self.first_data_sector + first_sector_of_cluster as usize
            }
        }
    }

    /// look for next cluster in FAT
    pub fn next_cluster(&self, cluster: Cluster) -> Result<Cluster, VolumeError> {
        let fat_offset = (cluster.0 * 2) as usize;
        let cur_fat_sector = self.fat_start + fat_offset / Block::SIZE;
        let offset = (fat_offset % Block::SIZE) as usize;
        let block = self.volume.read_block(cur_fat_sector).unwrap();
        let fat_entry = u16::from_le_bytes(
            block.inner()[offset..=offset + 1]
                .try_into()
                .unwrap_or([0; 2]),
        );
        match fat_entry {
            0xFFF7 => Err(VolumeError::BadCluster),         // Bad cluster
            0xFFF8..=0xFFFF => Err(VolumeError::EndOfFile), // There is no next cluster
            f => Ok(Cluster(f as u32)),                     // Seems legit
        }
    }

    /// Finds an entry in a given sector
    fn find_entry_in_sector(
        &self,
        match_name: &ShortFileName,
        sector: usize,
    ) -> Result<DirEntry, VolumeError> {
        let block = self.volume.read_block(sector).unwrap();
        for entry in 0..Block::SIZE / DirEntry::LEN {
            let start = entry * DirEntry::LEN;
            let end = (entry + 1) * DirEntry::LEN;
            let dir_entry = DirEntry::parse(&block.inner()[start..end])
                .map_err(|_| VolumeError::InvalidOperation)?;
            trace!("matching {} to {}...", dir_entry.filename(), match_name);
            if dir_entry.is_eod() {
                // Can quit early
                return Err(VolumeError::FileNotFound);
            } else if dir_entry.filename.matches(match_name) {
                // Found it
                return Ok(dir_entry);
            };
        }
        Err(VolumeError::NotInSector)
    }
}
