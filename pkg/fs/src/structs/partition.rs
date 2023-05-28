//! Partition Metadata
//!
//! This struct represents partitions' metadata.

use crate::alloc::borrow::ToOwned;

pub struct MBRPartitions {
    pub partitions: [PartitionMetaData; 4],
}

impl MBRPartitions {
    pub fn parse(data: &[u8; 512]) -> Self {
        let mut partitions = vec![PartitionMetaData::default(); 4];
        for i in 0..4 {
            partitions[i] = PartitionMetaData::parse(
                &data[0x1be + (i * 16)..0x1be + (i * 16) + 16]
                    .try_into()
                    .unwrap(),
            );
            if partitions[i].is_active() {
                trace!("Partition {}: {:?}", i, partitions[i]);
            }
        }
        Self {
            partitions: partitions.try_into().unwrap(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct PartitionMetaData {
    data: [u8; 16],
}

impl PartitionMetaData {
    /// Attempt to parse a Boot Parameter Block from a 512 byte sector.
    pub fn parse(data: &[u8; 16]) -> PartitionMetaData {
        PartitionMetaData {
            data: data.to_owned(),
        }
    }

    define_field!(u8, 0x00, status);
    define_field!(u8, 0x01, begin_head);
    // 0x02 - 0x03 begin sector & begin cylinder
    define_field!(u8, 0x04, filesystem_flag);
    define_field!(u8, 0x05, end_head);
    // 0x06 - 0x07 end sector & end cylinder
    define_field!(u32, 0x08, begin_lba);
    define_field!(u32, 0x0c, total_lba);

    pub fn is_active(&self) -> bool {
        self.status() == 0x80
    }

    pub fn is_extended(&self) -> bool {
        self.filesystem_flag() == 0x05
    }

    pub fn begin_sector(&self) -> u8 {
        self.data[2] & 0x3f
    }

    pub fn begin_cylinder(&self) -> u16 {
        (self.data[2] as u16 & 0xc0) << 2 | (self.data[3] as u16)
    }

    pub fn end_sector(&self) -> u8 {
        self.data[6] & 0x3f
    }

    pub fn end_cylinder(&self) -> u16 {
        (self.data[6] as u16 & 0xc0) << 2 | (self.data[7] as u16)
    }
}

impl core::fmt::Debug for PartitionMetaData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Partition Meta Data: {{")?;
        writeln!(f, "  Active: {}", self.is_active())?;
        writeln!(f, "  Begin Head: 0x{:02x}", self.begin_head())?;
        writeln!(f, "  Begin Sector: 0x{:04x}", self.begin_sector())?;
        writeln!(f, "  Begin Cylinder: 0x{:04x}", self.begin_cylinder())?;
        writeln!(f, "  Filesystem Flag: 0x{:02x}", self.filesystem_flag())?;
        writeln!(f, "  End Head: 0x{:02x}", self.end_head())?;
        writeln!(f, "  End Sector: 0x{:04x}", self.end_sector())?;
        writeln!(f, "  End Cylinder: 0x{:04x}", self.end_cylinder())?;
        writeln!(f, "  Begin LBA: 0x{:08x}", self.begin_lba())?;
        writeln!(f, "  Total LBA: 0x{:08x}", self.total_lba())?;
        write!(f, "}}")
    }
}
