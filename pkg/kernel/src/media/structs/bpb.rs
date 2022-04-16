//! BIOS Parameter Block
//!
//! reference:
//! - https://en.wikipedia.org/wiki/BIOS_parameter_block
//! - https://wiki.osdev.org/FAT#Boot_Record
//!
//! - https://github.com/xfoxfu/rust-xos/blob/main/fatpart/src/struct/bpb.rs
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/fat.rs

/// Indentifies the supported types of FAT format
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FatType {
    /// FAT16 Format
    Fat16,
    /// FAT32 Format
    Fat32,
}

/// Represents a Boot Parameter Block. This is the first sector of a FAT
/// formatted partition, and it describes various properties of the FAT
/// filesystem.
pub struct Bpb<'a> {
    data: &'a [u8; 512],
    fat_type: FatType
}

impl<'a> Bpb<'a> {
    /// Attempt to parse a Boot Parameter Block from a 512 byte sector.
    pub fn create_from_bytes(data: &[u8; 512]) -> Result<Bpb, &'static str> {
        let mut bpb = Bpb {
            data,
            fat_type: FatType::Fat16
        };

        if bpb.trail() != 0xAA55 {
            return Err("Bad BPB footer");
        }

        

        match bpb.fat_type {
            FatType::Fat16 => Ok(bpb),
            FatType::Fat32 if bpb.fs_ver() == 0 => {
                // Only support FAT32 version 0.0
                Ok(bpb)
            }
            _ => Err("Invalid FAT format"),
        }
    }

    // FAT16/FAT32
    define_field!(u16, 0x0b, bytes_per_sector);
    define_field!(u8 , 0x0d, sectors_per_cluster);
    define_field!(u16, 0x0e, reserved_sector_count);
    define_field!(u8 , 0x10, fat_count);
    define_field!(u16, 0x11, root_entries_count);
    define_field!(u16, 0x13, total_sectors_16);
    define_field!(u8 , 0x15, media_descriptor);
    define_field!(u16, 0x16, fat_size_16);
    define_field!(u16, 0x18, sectors_per_track);
    define_field!(u16, 0x1a, track_count);
    define_field!(u32, 0x1c, hidden_sectors);
    define_field!(u32, 0x20, total_sectors_32);

    // FAT32 only
    define_field!(u32, 0x24, fat_size_32);
    define_field!(u16, 0x2a, fs_ver);
    define_field!(u32, 0x2c, root_dir_cluster);
    define_field!(u16, 0x30, fs_info);
    define_field!(u16, 0x32, backup_boot_block);

    // Footer
    define_field!(u16, 0x1fe, trail);
}

#[test]
fn test_bpb() {
    // Taken from a Raspberry Pi bootable SD-Card
    const BPB_EXAMPLE: [u8; 512] = hex!(
        "EB 3C 90 6D 6B 66 73 2E 66 61 74 00 02 10 01 00
         02 00 02 00 00 F8 20 00 3F 00 FF 00 00 00 00 00
         00 E0 01 00 80 01 29 BB B0 71 77 62 6F 6F 74 20
         20 20 20 20 20 20 46 41 54 31 36 20 20 20 0E 1F
         BE 5B 7C AC 22 C0 74 0B 56 B4 0E BB 07 00 CD 10
         5E EB F0 32 E4 CD 16 CD 19 EB FE 54 68 69 73 20
         69 73 20 6E 6F 74 20 61 20 62 6F 6F 74 61 62 6C
         65 20 64 69 73 6B 2E 20 20 50 6C 65 61 73 65 20
         69 6E 73 65 72 74 20 61 20 62 6F 6F 74 61 62 6C
         65 20 66 6C 6F 70 70 79 20 61 6E 64 0D 0A 70 72
         65 73 73 20 61 6E 79 20 6B 65 79 20 74 6F 20 74
         72 79 20 61 67 61 69 6E 20 2E 2E 2E 20 0D 0A 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
         00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 AA"
    );
    let bpb = Bpb::create_from_bytes(&BPB_EXAMPLE).unwrap();
    assert_eq!(bpb.footer(), Bpb::FOOTER_VALUE);
    assert_eq!(bpb.oem_name(), b"mkfs.fat");
    assert_eq!(bpb.bytes_per_block(), 512);
    assert_eq!(bpb.blocks_per_cluster(), 16);
    assert_eq!(bpb.reserved_block_count(), 1);
    assert_eq!(bpb.num_fats(), 2);
    assert_eq!(bpb.root_entries_count(), 512);
    assert_eq!(bpb.total_blocks16(), 0);
    assert_eq!(bpb.fat_size16(), 32);
    assert_eq!(bpb.total_blocks32(), 122_880);
    assert_eq!(bpb.footer(), 0xAA55);
    assert_eq!(bpb.volume_label(), b"boot       ");
    assert_eq!(bpb.fat_size(), 32);
    assert_eq!(bpb.total_blocks(), 122_880);
    assert_eq!(bpb.fat_type, FatType::Fat16);
}
