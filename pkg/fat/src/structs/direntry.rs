//! Directory Entry
//!
//! reference:
//! - https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32
//! - https://github.com/xfoxfu/rust-xos/blob/main/fatpart/src/struct/dir_entry.rs
//! - https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/src/filesystem.rs

use chrono::{DateTime, Utc, TimeZone};
use bitflags::bitflags;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DirEntry {
    pub filename: ShortFileName,
    pub moditified_time: DateTime<Utc>,
    pub created_time: DateTime<Utc>,
    pub accessed_time: DateTime<Utc>,
    pub cluster: u32,
    pub attributes: Attributes,
    pub size: u32,
}

bitflags! {
    /// File Attributes
    pub struct Attributes: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN    = 0x02;
        const SYSTEM    = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE   = 0x20;
        const LFN       = 0x0f; // Long File Name
    }
}


impl DirEntry {
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

    pub fn parse(data: &[u8]) -> Result<DirEntry, FilenameError> {
        let pos = data.iter().position(|&x| x == 0).unwrap_or(data.len());
        // println!("pos: {:?}", pos);

        let filename = ShortFileName::new(&data[..pos]);

        // TODO: parse long file name

        if filename.is_eod() || filename.is_unused() {
            return Err(FilenameError::UnableToParse);
        }

        let attributes = Attributes::from_bits_truncate(data[11]);
        // 12: Reserved. Must be set to zero
        // 13: CrtTimeTenth, not supported, set to zero
        let mut time = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);
        let created_time = prase_datetime(time);

        time = u32::from_le_bytes([0, 0, data[18], data[19]]);
        let accessed_time = prase_datetime(time);

        let cluster = (data[27] as u32) << 8  | (data[26] as u32) |
                           (data[20] as u32) << 16 | (data[21] as u32) << 24;

        time = u32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        let moditified_time = prase_datetime(time);

        let size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        Ok(DirEntry {
            filename,
            moditified_time,
            created_time,
            accessed_time,
            cluster,
            attributes,
            size,
        })
    }
}

fn prase_datetime(time: u32) -> DateTime<Utc> {
    let year = ((time >> 25) + 1980) as i32;
    let month = (time >> 21) & 0x0f;
    let day = (time >> 16) & 0x1f;
    let hour = (time >> 11) & 0x1f;
    let minute = (time >> 5) & 0x3f;
    let second = (time & 0x1f) * 2;

    // trace!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);
    Utc.ymd(year, month, day).and_hms(hour, minute, second)
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

    pub fn parse(name: &str) -> Result<ShortFileName, FilenameError> {
        let mut sfn = ShortFileName { name: [0; 8], ext: [0; 3] };
        let mut idx = 0;
        let mut seen_dot = false;
        for ch in name.bytes() {
            match ch {
                // Microsoft say these are the invalid characters
                0x00..=0x1F | 0x20 | 0x22 | 0x2A | 0x2B | 0x2C | 0x2F | 0x3A | 0x3B
                     | 0x3C | 0x3D | 0x3E | 0x3F | 0x5B | 0x5C | 0x5D | 0x7C => {
                    return Err(FilenameError::InvalidCharacter);
                }
                // Denotes the start of the file extension
                b'.' => {
                    if idx >= 1 && idx <= 8 {
                        seen_dot = true;
                    } else {
                        return Err(FilenameError::MisplacedPeriod);
                    }
                }
                _ => {
                    let ch = ch.to_ascii_uppercase();
                    if seen_dot {
                        if idx >= 8 && idx < 11 {
                            sfn.ext[idx - 8] = ch;
                        } else {
                            return Err(FilenameError::NameTooLong);
                        }
                    } else if idx < 8 {
                        sfn.name[idx] = ch;
                    } else {
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
        if self.ext == [0, 0, 0] {
            write!(f, "{}", self.basename())
        } else {
            write!(f, "{}.{}", self.basename(), self.extension())
        }
    }
}

impl core::fmt::Display for ShortFileName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut length = 1;
        for &c in self.name.iter() {
            if c != b' ' && c != 0 {
                write!(f, "{}", c as char)?;
                length += 1;
            }
        }
        write!(f, ".")?;
        for &c in self.ext.iter() {
            if c != b' ' && c != 0 {
                write!(f, "{}", c as char)?;
                length += 1;
            }
        }
        if let Some(mut width) = f.width() {
            if width > length {
                width -= length;
                for _ in 0..width {
                    write!(f, "{}", f.fill())?;
                }
            }
        }
        Ok(())
    }
}

/// Various filename related errors that can occur.
#[derive(Debug, Clone)]
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
