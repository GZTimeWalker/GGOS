//! File
//!
//! reference:
//! - <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>
//! - <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs>

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file.
    pub offset: usize,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file.
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
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

impl Seek for File {
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

impl Read for File {
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
                .inner
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

impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}
