use crate::ata::*;
use alloc::{borrow::ToOwned, string::ToString};
use fs::{device::BlockDevice, *};
use spin::Mutex;

pub type Disk = fs::device::Disk<'static, MutexDrive<'static>>;
pub type Volume = fs::device::FAT16Volume<'static, MutexDrive<'static>>;

once_mutex!(DRIVE: Drive);
guard_access_fn!(get_drive(DRIVE: Drive));

pub static FILESYSTEM: spin::Once<Volume> = spin::Once::new();

fn fs() -> &'static Volume {
    FILESYSTEM.get().unwrap()
}

pub struct MutexDrive<'a>(pub &'a Mutex<Drive>);

static MUTEX_DRIVE: spin::Once<MutexDrive> = spin::Once::new();

pub fn get_device() -> &'static MutexDrive<'static> {
    MUTEX_DRIVE.get().unwrap()
}

impl<'a> Device<Block> for MutexDrive<'a> {
    fn read(&self, buf: &mut [Block], offset: usize, size: usize) -> Result<usize, DeviceError> {
        self.0.try_lock().unwrap().read(buf, offset, size)
    }
}

impl<'a> BlockDevice for MutexDrive<'a> {
    fn block_count(&self) -> Result<usize, DeviceError> {
        self.0.try_lock().unwrap().block_count()
    }

    fn read_block(&self, offset: usize) -> Result<Block, DeviceError> {
        self.0.try_lock().unwrap().read_block(offset)
    }
}

pub fn init() {
    info!("Initializing filesystem...");

    init_DRIVE(Drive::open(0, 0).unwrap());
    MUTEX_DRIVE.call_once(|| MutexDrive(DRIVE.get().unwrap()));
    let [p0, _, _, _] = Disk::new(get_device()).volumes();
    FILESYSTEM.call_once(|| FAT16Volume::new(p0));

    info!("Initialized Filesystem.");
}

pub fn ls(path_ori: &str) {
    let mut path = path_ori.to_owned();
    let mut root = fs::root_dir();

    while let Some(pos) = path.find('/') {
        let dir = path[..pos].to_owned();

        let tmp = fs::find_directory_entry(
            fs(), &root, dir.as_str()
        );

        if tmp.is_err() {
            error!("Directory not found: {}, {:?}", path_ori, tmp);
            return;
        }

        root = tmp.unwrap();

        path = path[pos + 1..].to_string();

        if path.len() == 0 {
            break;
        }
    }

    println!("     Size | Last Modified       | Name");

    if let Err(err) = fs::iterate_dir(fs(), &root, |entry| {
        println!("{}", entry);
    }) {
        println!("{:?}", err);
    }
}

pub fn cat(path: &str) {
    let root = fs::root_dir();
    let file = fs::open_file(fs(), &root, path, file::Mode::ReadOnly);

    if file.is_err() {
        println!("ERROR: {:?}", file.unwrap_err());
        return;
    }

    let file = file.unwrap();

    let data = fs::read(fs(), &file);

    if data.is_err() {
        println!("ERROR: {:?}", data.unwrap_err());
        return;
    }

    let data = data.unwrap();

    let mut count = 0;
    print!("    ");
    for (idx, b) in data.iter().enumerate() {
        print!("{:02x}", b);
        count += 1;
        if count % 8 == 0 {
            print!(" ");
        }
        if count == 24 {
            print!(" | ");
            for i in idx - 23..=idx {
                let d = data[i];
                if (d as char).is_ascii_graphic() || d == 0x20 {
                    print!("{}", d as char);
                } else {
                    print!(".");
                }
            }
            println!();
            count = 0;
            print!("    ")
        }
    }
    if count > 0 {
        for _ in count..24 {
            print!("  ");
        }
        for _ in 0..3 - (count / 8) {
            print!(" ");
        }
        print!(" | ");
        for i in data.len() - count..data.len() {
            let d = data[i];
            if (d as char).is_ascii_graphic() || d == 0x20 {
                print!("{}", d as char);
            } else {
                print!(".");
            }
        }
    }
    println!();
}
