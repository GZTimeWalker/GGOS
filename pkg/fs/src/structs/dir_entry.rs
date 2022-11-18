//! Directory Entry
//!
//! reference:
//! - <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>
//! - <https://github.com/xfoxfu/rust-xos/blob/main/fatpart/src/struct/dir_entry.rs>
//! - <https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs>

use alloc::string::String;
use bitflags::bitflags;
use chrono::LocalResult::Single;
use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DirEntry {
    pub filename: ShortFileName,
    pub moditified_time: DateTime<Utc>,
    pub created_time: DateTime<Utc>,
    pub accessed_time: DateTime<Utc>,
    pub cluster: Cluster,
    pub attributes: Attributes,
    pub size: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cluster(pub u32);

bitflags! {
    /// File Attributes
    pub struct Attributes: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN    = 0x02;
        const SYSTEM    = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE   = 0x20;
        const LFN       = 0x0f; // Long File Name, Not Implemented
    }
}

impl DirEntry {
    pub const LEN: usize = 0x20;

    pub fn is_readonly(&self) -> bool {
        self.attributes.contains(Attributes::READ_ONLY)
    }

    pub fn is_hidden(&self) -> bool {
        self.attributes.contains(Attributes::HIDDEN)
    }

    pub fn is_system(&self) -> bool {
        self.attributes.contains(Attributes::SYSTEM)
    }

    pub fn is_volume_id(&self) -> bool {
        self.attributes.contains(Attributes::VOLUME_ID)
    }

    pub fn is_directory(&self) -> bool {
        self.attributes.contains(Attributes::DIRECTORY)
    }

    pub fn is_archive(&self) -> bool {
        self.attributes.contains(Attributes::ARCHIVE)
    }

    pub fn is_long_name(&self) -> bool {
        self.attributes.contains(Attributes::LFN)
    }

    pub fn is_eod(&self) -> bool {
        self.filename.is_eod()
    }

    pub fn is_unused(&self) -> bool {
        self.filename.is_unused()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_eod() && !self.is_unused()
    }

    pub fn filename(&self) -> String {
        if self.is_valid() && !self.is_long_name() {
            format!("{}", self.filename)
        } else {
            String::from("unknown")
        }
    }

    /// For Standard 8.3 format
    pub fn parse(data: &[u8]) -> Result<DirEntry, FilenameError> {
        trace!(
            "parsing file...\n    {:016x} {:016x} {:016x} {:016x}",
            u64::from_be_bytes(data[0..8].try_into().unwrap()),
            u64::from_be_bytes(data[8..16].try_into().unwrap()),
            u64::from_be_bytes(data[16..24].try_into().unwrap()),
            u64::from_be_bytes(data[24..32].try_into().unwrap()),
        );

        let filename = ShortFileName::new(&data[..11]);

        // TODO: parse long file name
        // if filename.is_eod() || filename.is_unused() {
        //     return Err(FilenameError::UnableToParse);
        // }

        let attributes = Attributes::from_bits_truncate(data[11]);

        // 12: Reserved. Must be set to zero
        // 13: CrtTimeTenth, not supported, set to zero

        let mut time = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);
        let created_time = prase_datetime(time);

        time = u32::from_le_bytes([0, 0, data[18], data[19]]);
        let accessed_time = prase_datetime(time);

        let cluster = (data[27] as u32) << 8
            | (data[26] as u32) << 0
            | (data[21] as u32) << 24
            | (data[20] as u32) << 16;

        time = u32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        let moditified_time = prase_datetime(time);

        let size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        Ok(DirEntry {
            filename,
            moditified_time,
            created_time,
            accessed_time,
            cluster: Cluster(cluster),
            attributes,
            size,
        })
    }

    fn humanized_size(&self) -> (f32, String) {
        let bytes = self.size as f32;
        if bytes < 1024f32 {
            (bytes, String::from("B"))
        } else if (bytes / (1 << 10) as f32) < 1024f32 {
            (bytes / (1 << 10) as f32, String::from("K"))
        } else if (bytes / (1 << 20) as f32) < 1024f32 {
            (bytes / (1 << 20) as f32, String::from("M"))
        } else {
            (bytes / (1 << 30) as f32, String::from("G"))
        }
    }
}

fn prase_datetime(time: u32) -> DateTime<Utc> {
    let year = ((time >> 25) + 1980) as i32;
    let month = (time >> 21) & 0x0f;
    let day = (time >> 16) & 0x1f;
    let hour = (time >> 11) & 0x1f;
    let min = (time >> 5) & 0x3f;
    let sec = (time & 0x1f) * 2;

    if let Single(time) = Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
    {
        time
    } else {
        Utc.with_ymd_and_hms(1980, 1, 1, 0, 0, 0).single().unwrap()
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct ShortFileName {
    pub name: [u8; 8],
    pub ext: [u8; 3],
}

impl ShortFileName {
    pub fn new(buf: &[u8]) -> Self {
        Self {
            name: buf[..8].try_into().unwrap(),
            ext: buf[8..11].try_into().unwrap(),
        }
    }

    pub fn basename(&self) -> &str {
        core::str::from_utf8(&self.name).unwrap()
    }

    pub fn extension(&self) -> &str {
        core::str::from_utf8(&self.ext).unwrap()
    }

    pub fn is_eod(&self) -> bool {
        self.name[0] == 0x00 && self.ext[0] == 0x00
    }

    pub fn is_unused(&self) -> bool {
        self.name[0] == 0xE5
    }

    pub fn matches(&self, sfn: &ShortFileName) -> bool {
        self.name == sfn.name && self.ext == sfn.ext
    }

    pub fn parse(name: &str) -> Result<ShortFileName, FilenameError> {
        let mut sfn = ShortFileName {
            name: [0x20; 8],
            ext: [0x20; 3],
        };
        let mut idx = 0;
        let mut seen_dot = false;
        for ch in name.bytes() {
            match ch {
                // Microsoft say these are the invalid characters
                0x00..=0x1F
                | 0x20
                | 0x22
                | 0x2A
                | 0x2B
                | 0x2C
                | 0x2F
                | 0x3A
                | 0x3B
                | 0x3C
                | 0x3D
                | 0x3E
                | 0x3F
                | 0x5B
                | 0x5C
                | 0x5D
                | 0x7C => {
                    return Err(FilenameError::InvalidCharacter);
                }
                // Denotes the start of the file extension
                b'.' => {
                    if idx >= 1 && idx <= 8 {
                        seen_dot = true;
                        idx = 8;
                    } else {
                        return Err(FilenameError::MisplacedPeriod);
                    }
                }
                _ => {
                    let ch = ch.to_ascii_uppercase();
                    trace!("char: '{}', at: {}", ch as char, idx);
                    if seen_dot {
                        if idx >= 8 && idx < 11 {
                            sfn.ext[idx - 8] = ch;
                        } else {
                            return Err(FilenameError::NameTooLong);
                        }
                    } else if idx < 8 {
                        sfn.name[idx] = ch;
                    } else {
                        trace!("1");
                        return Err(FilenameError::NameTooLong);
                    }
                    idx += 1;
                }
            }
        }
        if idx == 0 {
            return Err(FilenameError::FilenameEmpty);
        }
        Ok(sfn)
    }
}

impl core::fmt::Debug for ShortFileName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::fmt::Display for ShortFileName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.ext[0] == 0x20 {
            write!(f, "{}", self.basename().trim_end())
        } else {
            write!(
                f,
                "{}.{}",
                self.basename().trim_end(),
                self.extension().trim_end()
            )
        }
    }
}

impl core::fmt::Display for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(
            f,
            "{:>5.*}{} | {} | {}{}",
            1,
            size,
            unit,
            self.moditified_time.format("%Y/%m/%d %H:%M:%S"),
            self.filename,
            if self.is_directory() { "/" } else { "" }
        )
    }
}

impl Cluster {
    /// Magic value indicating an invalid cluster value.
    pub const INVALID: Cluster = Cluster(0xFFFF_FFF6);
    /// Magic value indicating a bad cluster.
    pub const BAD: Cluster = Cluster(0xFFFF_FFF7);
    /// Magic value indicating a empty cluster.
    pub const EMPTY: Cluster = Cluster(0x0000_0000);
    /// Magic value indicating the cluster holding the root directory (which
    /// doesn't have a number in FAT16 as there's a reserved region).
    pub const ROOT_DIR: Cluster = Cluster(0xFFFF_FFFC);
    /// Magic value indicating that the cluster is allocated and is the final cluster for the file
    pub const END_OF_FILE: Cluster = Cluster(0xFFFF_FFFF);
}

impl core::ops::Add<u32> for Cluster {
    type Output = Cluster;
    fn add(self, rhs: u32) -> Cluster {
        Cluster(self.0 + rhs)
    }
}

impl core::ops::AddAssign<u32> for Cluster {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl core::ops::Add<Cluster> for Cluster {
    type Output = Cluster;
    fn add(self, rhs: Cluster) -> Cluster {
        Cluster(self.0 + rhs.0)
    }
}

impl core::ops::AddAssign<Cluster> for Cluster {
    fn add_assign(&mut self, rhs: Cluster) {
        self.0 += rhs.0;
    }
}

impl core::fmt::Display for Cluster {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

impl core::fmt::Debug for Cluster {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

/// Various filename related errors that can occur.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilenameError {
    /// Tried to create a file with an invalid character.
    InvalidCharacter,
    /// Tried to create a file with no file name.
    FilenameEmpty,
    /// Given name was too long (we are limited to 8.3).
    NameTooLong,
    /// Can't start a file with a period, or after 8 characters.
    MisplacedPeriod,
    /// Can't extract utf8 from file name
    Utf8Error,
    /// Can't parse file entry
    UnableToParse,
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
