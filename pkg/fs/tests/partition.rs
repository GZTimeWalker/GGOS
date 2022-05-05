use ggfs::structs::partition::*;

#[cfg(test)]
#[test]
fn partition_test() {
    let meta0 = PartitionMetaData::parse(&[
        0x80, 0x01, 0x01, 0x00, 0x0B, 0xFE, 0xBF, 0xFC,
        0x3F, 0x00, 0x00, 0x00, 0x7E, 0x86, 0xBB, 0x00,
    ]).unwrap();

    println!("{:?}", meta0);

    assert_eq!(meta0.is_active(), true);
    assert_eq!(meta0.begin_head(), 1);
    assert_eq!(meta0.begin_sector(), 1);
    assert_eq!(meta0.begin_cylinder(), 0);
    assert_eq!(meta0.filesystem_flag(), 0x0b);
    assert_eq!(meta0.end_head(), 254);
    assert_eq!(meta0.end_sector(), 63);
    assert_eq!(meta0.end_cylinder(), 764);
    assert_eq!(meta0.begin_lba(), 63);
    assert_eq!(meta0.total_lba(), 12289662);
}
