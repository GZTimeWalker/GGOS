use super::bpb::FAT16Bpb;
use core::ops::Range;

pub struct FAT16Table<'a> {
    data: &'a [u8],
    bpb: &'a FAT16Bpb,
}

impl<'a> FAT16Table<'a> {
    pub fn new(bpb: &'a FAT16Bpb, data: &'a [u8]) -> Result<Self, usize> {
        if data.len() < (bpb.sectors_per_fat() * bpb.bytes_per_sector()) as usize {
            Err(data.len())?
        }

        Ok(Self { data, bpb })
    }

    /// 获取第 id 个 FAT 表项对应的扇区范围
    pub fn cluster_sector(&self, id: u16) -> Range<u16> {
        let start = self.bpb.reserved_sector_count()
            + self.bpb.fat_count() as u16 * self.bpb.sectors_per_fat()
            + self.bpb.sectors_per_cluster() as u16 * id;
        let end = start + self.bpb.sectors_per_cluster() as u16;
        start..end
    }

    /// 获取第 id 个 FAT 表项的下一个 FAT 表项
    pub fn next_cluster(&self, id: u16) -> Option<u16> {
        let raw = u16::from_le_bytes([data[2 * id], data[2 * id + 1]]);
        if raw > 0x0001 && raw < 0xFFF0 {
            Some(raw)
        } else {
            None
        }
    }
}
