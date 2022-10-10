//! FAT16 BIOS Parameter Block
//!
//! reference:
//! - <https://en.wikipedia.org/wiki/BIOS_parameter_block>
//! - <https://wiki.osdev.org/FAT#Boot_Record>
//! - <https://github.com/xfoxfu/rust-xos/blob/main/fatpart/src/struct/bpb.rs>
//! - <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs>

/// Represents a Boot Parameter Block. This is the first sector of a FAT 16
/// formatted partition, and it describes various properties of the FAT 16
/// filesystem.
pub struct FAT16Bpb {
    data: [u8; 512],
}

impl FAT16Bpb {
    /// Attempt to parse a Boot Parameter Block from a 512 byte sector.
    pub fn new(data: &[u8]) -> Result<FAT16Bpb, &'static str> {
        let data = data.try_into().unwrap();
        let bpb = FAT16Bpb { data };

        if bpb.data.len() != 512 || bpb.trail() != 0xAA55 {
            return Err("Bad BPB format");
        }

        Ok(bpb)
    }

    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16() == 0 {
            self.total_sectors_32()
        } else {
            self.total_sectors_16() as u32
        }
    }

    define_field!([u8; 8], 0x03, oem_name);
    define_field!(u16, 0x0b, bytes_per_sector);
    define_field!(u8, 0x0d, sectors_per_cluster);
    define_field!(u16, 0x0e, reserved_sector_count);
    define_field!(u8, 0x10, fat_count);
    define_field!(u16, 0x11, root_entries_count);
    define_field!(u16, 0x13, total_sectors_16);
    define_field!(u8, 0x15, media_descriptor);
    define_field!(u16, 0x16, sectors_per_fat);
    define_field!(u16, 0x18, sectors_per_track);
    define_field!(u16, 0x1a, track_count);
    define_field!(u32, 0x1c, hidden_sectors);
    define_field!(u32, 0x20, total_sectors_32);
    define_field!(u8, 0x24, drive_number);
    define_field!(u8, 0x25, reserved_flags);
    define_field!(u8, 0x26, boot_signature);
    define_field!(u32, 0x27, volume_id);
    define_field!([u8; 11], 0x2b, volume_label);
    define_field!([u8; 8], 0x36, system_identifier);
    define_field!(u16, 0x1fe, trail);
}

impl core::fmt::Debug for FAT16Bpb {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FAT16 BPB: {{\n")?;
        write!(f, "  OEM Name: {:?}\n", self.oem_name_str())?;
        write!(f, "  Bytes per Sector: {}\n", self.bytes_per_sector())?;
        write!(f, "  Sectors per Cluster: {}\n", self.sectors_per_cluster())?;
        write!(
            f,
            "  Reserved Sector Count: {}\n",
            self.reserved_sector_count()
        )?;
        write!(f, "  FAT Count: {}\n", self.fat_count())?;
        write!(f, "  Root Entries Count: {}\n", self.root_entries_count())?;
        write!(f, "  Total Sectors: {}\n", self.total_sectors())?;
        write!(f, "  Media Descriptor: {}\n", self.media_descriptor())?;
        write!(f, "  Sectors per FAT: {}\n", self.sectors_per_fat())?;
        write!(f, "  Sectors per Track: {}\n", self.sectors_per_track())?;
        write!(f, "  Track Count: {}\n", self.track_count())?;
        write!(f, "  Hidden Sectors: {}\n", self.hidden_sectors())?;
        write!(f, "  Total Sectors: {}\n", self.total_sectors())?;
        write!(f, "  Drive Number: {}\n", self.drive_number())?;
        write!(f, "  Reserved Flags: {}\n", self.reserved_flags())?;
        write!(f, "  Boot Signature: {}\n", self.boot_signature())?;
        write!(f, "  Volume ID: {}\n", self.volume_id())?;
        write!(f, "  Volume Label: {:?}\n", self.volume_label_str())?;
        write!(
            f,
            "  System Identifier: {:?}\n",
            self.system_identifier_str()
        )?;
        write!(f, "  Trail: 0x{:04X}\n", self.trail())?;
        write!(f, "}}")
    }
}
