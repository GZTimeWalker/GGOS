//! File
//!
//! reference:
//! - https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs

use crate::dir_entry::*;

#[derive(Debug)]
pub struct File {
    /// The starting point of the file.
    start_cluster: Cluster,
    /// The current cluster
    current_cluster: Cluster,
    /// How far through the file we've read (in bytes).
    current_offset: u32,
    /// The length of the file, in bytes.
    length: u32,
    /// What mode the file was opened in
    mode: Mode,
    /// DirEntry of this file
    entry: DirEntry,
}

/// The different ways we can open a file.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    /// Open a file for reading, if it exists.
    ReadOnly,
    /// Open a file for appending (writing to the end of the existing file), if it exists.
    ReadWriteAppend,
    /// Open a file and remove all contents, before writing to the start of the existing file, if it exists.
    ReadWriteTruncate,
    /// Create a new empty file. Fail if it exists.
    ReadWriteCreate,
    /// Create a new empty file, or truncate an existing file.
    ReadWriteCreateOrTruncate,
    /// Create a new empty file, or append to an existing file.
    ReadWriteCreateOrAppend,
}

#[derive(Debug)]
pub enum FileError {
    OutOfBound(i32),
}

impl File {
    /// Are we at the end of the file?
    pub fn eof(&self) -> bool {
        self.current_offset == self.length
    }

    /// How long is the file?
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Seek to a new position in the file, relative to the start of the file.
    pub fn seek_from_start(&mut self, offset: u32) -> Result<(), FileError> {
        if offset <= self.length {
            self.current_offset = offset;
            if offset < self.current_cluster.0 {
                // Back to start
                self.current_cluster = self.start_cluster;
            }
            Ok(())
        } else {
            Err(FileError::OutOfBound(offset as i32))
        }
    }

    /// Seek to a new position in the file, relative to the end of the file.
    pub fn seek_from_end(&mut self, offset: u32) -> Result<(), FileError> {
        if offset <= self.length {
            self.current_offset = self.length - offset;
            if offset < self.current_cluster.0 {
                // Back to start
                self.current_cluster = self.start_cluster;
            }
            Ok(())
        } else {
            Err(FileError::OutOfBound(offset as i32))
        }
    }

    /// Seek to a new position in the file, relative to the current position.
    pub fn seek_from_current(&mut self, offset: i32) -> Result<(), FileError> {
        let new_offset = i64::from(self.current_offset) + i64::from(offset);
        if new_offset >= 0 && new_offset <= i64::from(self.length) {
            self.current_offset = new_offset as u32;
            Ok(())
        } else {
            Err(FileError::OutOfBound(offset))
        }
    }

    /// Amount of file left to read.
    pub fn left(&self) -> u32 {
        self.length - self.current_offset
    }
}
