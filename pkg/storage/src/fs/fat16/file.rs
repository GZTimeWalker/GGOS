//! File
//!
//! reference:
//! - <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>
//! - <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs>

use x86_64::registers::debug;

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file.
    pub offset: usize,
    /// DirEntry of this file
    entry: DirEntry,
    /// The current cluster of this file
    current: Cluster,
    /// The file system handle that contains this file.
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let length = self.length();

        if self.offset >= length {
            return Ok(0);
        }

        let sector_pre_cluster = self.handle.bpb.sectors_per_cluster() as usize;
        let sector_size = self.handle.bpb.bytes_per_sector() as usize;
        let cluster_size = sector_pre_cluster * sector_size;

        let mut block = Block::default();
        let mut bytes_read = 0;

        while bytes_read < buf.len() && self.offset < length {
            let cluster_sector = self.handle.cluster_to_sector(&self.current);
            let cluster_offset = self.offset % cluster_size;
            let current_sector = cluster_sector + cluster_offset / BLOCK_SIZE;

            self.handle.inner.read_block(current_sector, &mut block)?;

            let current_offset = self.offset % BLOCK_SIZE;
            let block_remain = BLOCK_SIZE - current_offset;
            let file_remain = length - self.offset;
            let buf_remain = buf.len() - bytes_read;
            let to_read = buf_remain.min(block_remain).min(file_remain);

            buf[bytes_read..bytes_read + to_read]
                .copy_from_slice(&block[current_offset..current_offset + to_read]);

            bytes_read += to_read;
            self.offset += to_read;

            if to_read < block_remain {
                break;
            }

            if self.offset % cluster_size == 0 {
                if let Ok(next_cluster) = self.handle.next_cluster(&self.current) {
                    self.current = next_cluster;
                } else {
                    break;
                }
            }
        }

        Ok(bytes_read)
    }
}

impl Seek for File {
    fn seek(&mut self, _pos: SeekFrom) -> Result<usize> {
        unimplemented!()
    }
}

impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}
