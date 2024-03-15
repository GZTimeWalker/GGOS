//! File
//!
//! reference:
//! - <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>
//! - <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs>

use super::*;
use num_enum::TryFromPrimitive;

#[derive(Debug, Clone)]
pub struct File<T>
where
    T: BlockDevice<Block512>,
{
    /// The current offset in the file.
    pub offset: usize,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file.
    handle: Fat16Handle<T>,
}

impl<T> File<T>
where
    T: BlockDevice<Block512>,
{
    pub fn new(handle: Fat16Handle<T>, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl<T> Seek for File<T>
where
    T: BlockDevice<Block512>,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        match pos {
            SeekFrom::Start(offset) => {
                self.offset = offset;
            }
            SeekFrom::End(offset) => {
                self.offset = (self.entry.size as isize + offset) as usize;
            }
            SeekFrom::Current(offset) => {
                self.offset = (self.offset as isize + offset) as usize;
            }
        }
        Ok(self.offset)
    }
}

impl<T> Read for File<T>
where
    T: BlockDevice<Block512>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let length = self.length();

        if self.offset >= length {
            return Ok(0);
        }

        let total_blocks = (length + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let mut current_block = self.offset / BLOCK_SIZE;
        let mut block = Block::default();
        let sector = self.handle.cluster_to_sector(&self.entry.cluster);

        let mut bytes_read = 0;

        while bytes_read < buf.len() && self.offset < length && current_block < total_blocks {
            current_block = self.offset / BLOCK_SIZE;
            let current_offset = self.offset % BLOCK_SIZE;
            self.handle
                .volume
                .read_block(sector + current_block, &mut block)?;

            let block_remain = BLOCK_SIZE - current_offset;
            let buf_remain = buf.len() - bytes_read;
            let file_remain = length - self.offset;
            let to_read = buf_remain.min(block_remain).min(file_remain);

            buf[bytes_read..bytes_read + to_read]
                .copy_from_slice(&block[current_offset..current_offset + to_read]);

            bytes_read += to_read;
            self.offset += to_read;
        }

        Ok(bytes_read)
    }
}
