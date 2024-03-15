use super::*;
use alloc::vec::Vec;

/// The `Read` trait allows for reading bytes from a source.
pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read all bytes until EOF in this source, placing them into `buf`.
    fn read_all(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut start_len = buf.len();
        loop {
            buf.resize(start_len + 512, 0);
            match self.read(&mut buf[start_len..]) {
                Ok(0) => {
                    buf.truncate(start_len);
                    return Ok(buf.len());
                }
                Ok(n) => {
                    buf.truncate(start_len + n);
                    start_len += n;
                }
                Err(e) => {
                    buf.truncate(start_len);
                    return Err(e);
                }
            }
        }
    }

    /// Read the exact number of bytes required to fill `buf`.
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(FsError::EndOfFile)
        } else {
            Ok(())
        }
    }
}

/// The `Write` trait allows for writing bytes to a source.
pub trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    fn flush(&mut self) -> Result<()>;

    /// Attempts to write an entire buffer into this writer.
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(FsError::WriteZero);
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(usize),

    /// Sets the offset to the size of this object plus the offset.
    End(isize),

    /// Sets the offset to the current position plus the offset.
    Current(isize),
}

/// The `Seek` trait provides a cursor within byte stream.
pub trait Seek {
    /// Seek to an offset, in bytes, in a stream.
    fn seek(&mut self, pos: SeekFrom) -> Result<usize>;
}

/// Trait combining Seek and Read, return value for opening files
pub trait SeekAndRead: Seek + Read {}

/// Trait combining Seek and Write, return value for writing files
pub trait SeekAndWrite: Seek + Write {}

impl<T> SeekAndRead for T where T: Seek + Read {}

impl<T> SeekAndWrite for T where T: Seek + Write {}
