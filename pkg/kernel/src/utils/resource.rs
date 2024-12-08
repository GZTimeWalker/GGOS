use alloc::{collections::BTreeMap, string::String};
use pc_keyboard::DecodedKey;
use spin::Mutex;
use storage::{random::Random, Device, FileHandle};

use crate::input::try_get_key;

#[derive(Debug, Clone)]
pub enum StdIO {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct ResourceSet {
    pub handles: BTreeMap<u8, Mutex<Resource>>,
}

impl Default for ResourceSet {
    fn default() -> Self {
        let mut res = Self {
            handles: BTreeMap::new(),
        };

        res.open(Resource::Console(StdIO::Stdin));
        res.open(Resource::Console(StdIO::Stdout));
        res.open(Resource::Console(StdIO::Stderr));

        res
    }
}

impl ResourceSet {
    pub fn open(&mut self, res: Resource) -> u8 {
        let fd = self.handles.len() as u8;
        self.handles.insert(fd, Mutex::new(res));
        fd
    }

    pub fn close(&mut self, fd: u8) -> bool {
        self.handles.remove(&fd).is_some()
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        if let Some(count) = self.handles.get(&fd).and_then(|h| h.lock().read(buf)) {
            count as isize
        } else {
            -1
        }
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        if let Some(count) = self.handles.get(&fd).and_then(|h| h.lock().write(buf)) {
            count as isize
        } else {
            -1
        }
    }
}

pub enum Resource {
    File(FileHandle),
    Console(StdIO),
    Random(Random),
    Null,
}

impl Resource {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        match self {
            Resource::File(file) => {
                let ret = file.read(buf);
                if let Err(e) = ret {
                    error!("Failed to read file: {:?}", e);
                    None
                } else {
                    Some(ret.unwrap())
                }
            }
            Resource::Console(stdio) => match stdio {
                &mut StdIO::Stdin => Some(if buf.len() < 4 {
                    0
                } else if let Some(DecodedKey::Unicode(k)) = try_get_key() {
                    let s = k.encode_utf8(buf);
                    s.len()
                } else {
                    0
                }),
                _ => Some(0),
            },
            Resource::Random(random) => Some(random.read(buf, 0, buf.len()).unwrap()),
            Resource::Null => Some(0),
        }
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        match self {
            Resource::File(_) => None,
            Resource::Console(stdio) => match *stdio {
                StdIO::Stdin => Some(0),
                StdIO::Stdout => {
                    print!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
                StdIO::Stderr => {
                    warn!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
            },
            Resource::Random(_) => Some(0),
            Resource::Null => Some(buf.len()),
        }
    }
}

impl core::fmt::Debug for Resource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Resource::File(h) => write!(f, "File({})", h.meta.name),
            Resource::Console(c) => write!(f, "Console({:?})", c),
            Resource::Random(_) => write!(f, "Random"),
            Resource::Null => write!(f, "Null"),
        }
    }
}
