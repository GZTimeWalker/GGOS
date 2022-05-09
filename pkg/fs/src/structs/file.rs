//! File
//!
//! reference:
//! - https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs

use crate::*;
#[derive(Debug, Clone)]
pub struct File {
    /// The starting point of the file.
    pub start_cluster: Cluster,
    /// The length of the file, in bytes.
    pub length: u32,
    /// What mode the file was opened in
    pub mode: Mode,
    /// DirEntry of this file
    pub entry: DirEntry,
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
    /// How long is the file?
    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn start_cluster(&self) -> Cluster {
        self.start_cluster
    }
}
