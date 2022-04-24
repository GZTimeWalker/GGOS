use crate::structs::*;

/// Identifies a FAT16 Volume on the disk.
pub struct FatVolume {
    pub table: FAT16Table,
    pub partition: MBRPartitions
}
