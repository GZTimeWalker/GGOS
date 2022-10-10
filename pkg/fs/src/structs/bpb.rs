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
        writeln!(f, "FAT16 BPB: {{")?;
        writeln!(f, "  OEM Name: {:?}", self.oem_name_str())?;
        writeln!(f, "  Bytes per Sector: {}", self.bytes_per_sector())?;
        writeln!(f, "  Sectors per Cluster: {}", self.sectors_per_cluster())?;
        writeln!(
            f,
            "  Reserved Sector Count: {}",
            self.reserved_sector_count()
        )?;
        writeln!(f, "  FAT Count: {}", self.fat_count())?;
        writeln!(f, "  Root Entries Count: {}", self.root_entries_count())?;
        writeln!(f, "  Total Sectors: {}", self.total_sectors())?;
        writeln!(f, "  Media Descriptor: {}", self.media_descriptor())?;
        writeln!(f, "  Sectors per FAT: {}", self.sectors_per_fat())?;
        writeln!(f, "  Sectors per Track: {}", self.sectors_per_track())?;
        writeln!(f, "  Track Count: {}", self.track_count())?;
        writeln!(f, "  Hidden Sectors: {}", self.hidden_sectors())?;
        writeln!(f, "  Total Sectors: {}", self.total_sectors())?;
        writeln!(f, "  Drive Number: {}", self.drive_number())?;
        writeln!(f, "  Reserved Flags: {}", self.reserved_flags())?;
        writeln!(f, "  Boot Signature: {}", self.boot_signature())?;
        writeln!(f, "  Volume ID: {}", self.volume_id())?;
        writeln!(f, "  Volume Label: {:?}", self.volume_label_str())?;
        writeln!(f, "  System Identifier: {:?}", self.system_identifier_str())?;
        writeln!(f, "  Trail: 0x{:04X}", self.trail())?;
        write!(f, "}}")
    }
}
