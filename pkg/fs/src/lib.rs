#![cfg_attr(not(test), no_std)]
#![allow(dead_code, unused_imports)]
#![cfg_attr(test, feature(concat_bytes))]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

#[macro_use]
pub mod common;
mod disk;
mod fs;

pub use common::*;
pub use disk::*;
pub use fs::*;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

// 1. The disk structure
// How to read a file from disk
//
//   - The disk is a collection of partitions.
//     MBR (Master Boot Record) is the first sector of the disk.
//     The MBR contains information about the partitions.
//
//     [ MBR | Partitions ] [ Partition 1 ] [ Partition 2 ] [ Partition 3 ] [ Partition 4 ]
//
// 2. The partition structure (in Fat16)
//
//    - The partition is a collection of clusters.
//     BPB (Boot Parameter Block) is the first sector of the partition.
//     The BPB contains information about the filesystem.
//
//     [ Fat16 BPB ] [ Data ]
