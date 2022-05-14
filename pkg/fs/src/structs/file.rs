//! File
//!
//! reference:
//! - https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs

use crate::*;
use num_enum::TryFromPrimitive;

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
#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum Mode {
    /// Open a file for reading, if it exists.
    #[num_enum(default)]
    ReadOnly = 0,
    /// Open a file for appending (writing to the end of the existing file), if it exists.
    ReadWriteAppend = 1,
    /// Open a file and remove all contents, before writing to the start of the existing file, if it exists.
    ReadWriteTruncate = 2,
    /// Create a new empty file. Fail if it exists.
    ReadWriteCreate = 3,
    /// Create a new empty file, or truncate an existing file.
    ReadWriteCreateOrTruncate = 4,
    /// Create a new empty file, or append to an existing file.
    ReadWriteCreateOrAppend = 5,
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
