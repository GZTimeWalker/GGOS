//! Directory
//!
//! reference:
//! - https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs

use crate::dir_entry::*;

#[derive(Debug)]
pub struct Directory {
    /// The starting point of the directory listing.
    pub cluster: Cluster,
    /// Dir Entry of this directory, None for the root directory
    pub entry: Option<DirEntry>,
}

impl Directory {
    /// Create a new directory from a cluster number.
    pub fn new(cluster: Cluster) -> Self {
        Directory {
            cluster,
            entry: None,
        }
    }
}
