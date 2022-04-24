#[macro_use]
mod macros;

pub mod bpb;
pub mod block;
pub mod partition;
pub mod dir_entry;
pub mod fat_table;

pub use bpb::FAT16Bpb;
pub use fat_table::FAT16Table;
