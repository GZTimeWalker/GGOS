use alloc::string::String;
use fs::{Device, File, Random};

use crate::filesystem::{get_volume, StdIO};

#[derive(Debug, Clone)]
pub enum Resource {
    File(File),
    Console(StdIO),
    Random(Random),
    Null,
}

impl Resource {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, ()> {
        match self {
            Resource::File(file) => fs::read_to_buf(get_volume(), file, buf).map_err(|_| ()),
            Resource::Console(stdio) => match stdio {
                &StdIO::Stdin => {
                    return if buf.len() < 4 {
                        Ok(0)
                    } else {
                        // TODO: get key async
                        Ok(0)
                    }
                }
                _ => Err(()),
            },
            Resource::Random(random) => Ok(random.read(buf, 0, buf.len()).unwrap_or(0)),
            Resource::Null => Ok(0),
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, ()> {
        match self {
            Resource::File(_) => unimplemented!(),
            Resource::Console(stdio) => match stdio {
                &StdIO::Stdin => Err(()),
                &StdIO::Stdout => {
                    print!("{}", String::from_utf8_lossy(buf));
                    Ok(buf.len())
                }
                &StdIO::Stderr => {
                    warn!("{}", String::from_utf8_lossy(buf));
                    Ok(buf.len())
                }
            },
            Resource::Random(_) => Ok(0),
            Resource::Null => Ok(0),
        }
    }
}
