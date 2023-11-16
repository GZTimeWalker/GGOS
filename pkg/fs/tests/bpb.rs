#![feature(concat_bytes)]

use ggfs::structs::bpb::*;

#[cfg(test)]
#[test]
fn test_fat16_bpb_1() {
    // Taken from a Raspberry Pi bootable SD-Card
    const DATA: [u8; 192] = hex_literal::hex!(
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
        72 79 20 61 67 61 69 6E 20 2E 2E 2E 20 0D 0A 00"
    );

    const PADDING: &[u8] = concat_bytes!(
        [0x00; 318],
        [0x55, 0xAA]
    );

    let mut bpb_data = DATA.to_vec();
    bpb_data.extend_from_slice(PADDING);

    let bpb = FAT16Bpb::new(&bpb_data).unwrap();

    assert_eq!(bpb.oem_name(), b"mkfs.fat");
    assert_eq!(bpb.bytes_per_sector(), 512);
    assert_eq!(bpb.sectors_per_cluster(), 16);
    assert_eq!(bpb.reserved_sector_count(), 1);
    assert_eq!(bpb.fat_count(), 2);
    assert_eq!(bpb.root_entries_count(), 512);
    assert_eq!(bpb.total_sectors_16(), 0);
    assert_eq!(bpb.media_descriptor(), 0xf8);
    assert_eq!(bpb.sectors_per_fat(), 32);
    assert_eq!(bpb.sectors_per_track(), 63);
    assert_eq!(bpb.track_count(), 255);
    assert_eq!(bpb.hidden_sectors(), 0);
    assert_eq!(bpb.total_sectors_32(), 0x1e000);
    assert_eq!(bpb.drive_number(), 128);
    assert_eq!(bpb.reserved_flags(), 1);
    assert_eq!(bpb.boot_signature(), 0x29);
    assert_eq!(bpb.volume_id(), 0x7771b0bb);
    assert_eq!(bpb.volume_label(), b"boot       ");
    assert_eq!(bpb.system_identifier(), b"FAT16   ");

    assert_eq!(bpb.total_sectors(), 0x1e000);

    println!("{:?}", bpb);
}

#[cfg(test)]
#[test]
fn test_fat16_bpb_2() {
    // Taken from a Raspberry Pi bootable SD-Card
    const DATA: [u8; 64] = hex_literal::hex!(
       "EB 3E 90 4D 53 57 49 4E 34 2E 31 00 02 10 01 00
        02 00 02 00 00 F8 FC 00 3F 00 10 00 3F 00 00 00
        C1 BF 0F 00 80 00 29 FD 1A BE FA 51 45 4D 55 20
        56 56 46 41 54 20 46 41 54 31 36 20 20 20 00 00"
    );

    const PADDING: &[u8] = concat_bytes!(
        [0x00; 446],
        [0x55, 0xAA]
    );


    let mut bpb_data = DATA.to_vec();
    bpb_data.extend_from_slice(PADDING);

    let bpb = FAT16Bpb::new(&bpb_data).unwrap();

    assert_eq!(bpb.oem_name(), b"MSWIN4.1");
    assert_eq!(bpb.oem_name_str(), "MSWIN4.1");
    assert_eq!(bpb.bytes_per_sector(), 512);
    assert_eq!(bpb.sectors_per_cluster(), 16);
    assert_eq!(bpb.reserved_sector_count(), 1);
    assert_eq!(bpb.fat_count(), 2);
    assert_eq!(bpb.root_entries_count(), 512);
    assert_eq!(bpb.total_sectors_16(), 0);
    assert_eq!(bpb.media_descriptor(), 0xf8);
    assert_eq!(bpb.sectors_per_fat(), 0xfc);
    assert_eq!(bpb.sectors_per_track(), 63);
    assert_eq!(bpb.track_count(), 16);
    assert_eq!(bpb.hidden_sectors(), 63);
    assert_eq!(bpb.total_sectors_32(), 0xfbfc1);
    assert_eq!(bpb.drive_number(), 128);
    assert_eq!(bpb.reserved_flags(), 0);
    assert_eq!(bpb.boot_signature(), 0x29);
    assert_eq!(bpb.volume_id(), 0xfabe1afd);
    assert_eq!(bpb.volume_label(), b"QEMU VVFAT ");
    assert_eq!(bpb.volume_label_str(), "QEMU VVFAT ");
    assert_eq!(bpb.system_identifier(), b"FAT16   ");
    assert_eq!(bpb.system_identifier_str(), "FAT16   ");

    assert_eq!(bpb.total_sectors(), 0xfbfc1);

    println!("{:?}", bpb);
}

#[cfg(test)]
#[test]
fn test_fat16_bpb_3() {
    // Taken from a Raspberry Pi bootable SD-Card
    const BPB_EXAMPLE: [u8; 512] = hex_literal::hex!(
        "EB 3C 90 50 4B 57 49 4E 34 2E 31 00 02 40 01 00
        02 00 02 00 00 F8 A0 00 3F 00 FF 00 00 70 48 74
        AF ED 27 00 80 00 29 B7 06 BA 0E 4E 4F 20 4E 41
        4D 45 20 20 20 20 46 41 54 31 36 20 20 20 33 C9
        8E D1 BC F0 7B 8E D9 B8 00 20 8E C0 FC BD 00 7C
        38 4E 24 7D 24 8B C1 99 E8 3C 01 72 1C 83 EB 3A
        66 A1 1C 7C 26 66 3B 07 26 8A 57 FC 75 06 80 CA
        02 88 56 02 80 C3 10 73 EB 33 C9 8A 46 10 98 F7
        66 16 03 46 1C 13 56 1E 03 46 0E 13 D1 8B 76 11
        60 89 46 FC 89 56 FE B8 20 00 F7 E6 8B 5E 0B 03
        C3 48 F7 F3 01 46 FC 11 4E FE 61 BF 00 00 E8 E6
        00 72 39 26 38 2D 74 17 60 B1 0B BE A1 7D F3 A6
        61 74 32 4E 74 09 83 C7 20 3B FB 72 E6 EB DC A0
        FB 7D B4 7D 8B F0 AC 98 40 74 0C 48 74 13 B4 0E
        BB 07 00 CD 10 EB EF A0 FD 7D EB E6 A0 FC 7D EB
        E1 CD 16 CD 19 26 8B 55 1A 52 B0 01 BB 00 00 E8
        3B 00 72 E8 5B 8A 56 24 BE 0B 7C 8B FC C7 46 F0
        3D 7D C7 46 F4 29 7D 8C D9 89 4E F2 89 4E F6 C6
        06 96 7D CB EA 03 00 00 20 0F B6 C8 66 8B 46 F8
        66 03 46 1C 66 8B D0 66 C1 EA 10 EB 5E 0F B6 C8
        4A 4A 8A 46 0D 32 E4 F7 E2 03 46 FC 13 56 FE EB
        4A 52 50 06 53 6A 01 6A 10 91 8B 46 18 96 92 33
        D2 F7 F6 91 F7 F6 42 87 CA F7 76 1A 8A F2 8A E8
        C0 CC 02 0A CC B8 01 02 80 7E 02 0E 75 04 B4 42
        8B F4 8A 56 24 CD 13 61 61 72 0B 40 75 01 42 03
        5E 0B 49 75 06 F8 C3 41 BB 00 00 60 66 6A 00 EB
        B0 4E 54 4C 44 52 20 20 20 20 20 20 0D 0A 52 65
        6D 6F 76 65 20 64 69 73 6B 73 20 6F 72 20 6F 74
        68 65 72 20 6D 65 64 69 61 2E FF 0D 0A 44 69 73
        6B 20 65 72 72 6F 72 FF 0D 0A 50 72 65 73 73 20
        61 6E 79 20 6B 65 79 20 74 6F 20 72 65 73 74 61
        72 74 0D 0A 00 00 00 00 00 00 00 AC CB D8 55 AA"
    );

    let bpb = FAT16Bpb::new(&BPB_EXAMPLE).unwrap();

    println!("{:?}", bpb);
}
