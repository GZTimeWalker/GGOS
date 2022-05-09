use crate::ata::*;
use alloc::{borrow::ToOwned, string::ToString};
use fs::*;

pub type Disk = fs::device::Disk<Drive>;
pub type Volume = fs::device::FAT16Volume<Drive>;

pub static FILESYSTEM: spin::Once<Volume> = spin::Once::new();

pub fn get_volume() -> &'static Volume {
    FILESYSTEM.get().unwrap()
}

#[derive(Debug, Clone)]
pub struct StdIO;

impl StdIO {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn init() {
    info!("Initializing filesystem...");

    let [p0, _, _, _] = Disk::new(Drive::open(0, 0).unwrap()).volumes();
    FILESYSTEM.call_once(|| FAT16Volume::new(p0));

    info!("Initialized Filesystem.");
}

fn resolve_path(root_path: &str) -> Option<Directory> {
    let mut path = root_path.to_owned();
    let mut root = fs::root_dir();

    while let Some(pos) = path.find('/') {
        let dir = path[..pos].to_owned();

        let tmp = fs::find_directory_entry(
            get_volume(), &root, dir.as_str()
        );

        if tmp.is_err() {
            warn!("Directory not found: {}, {:?}", root_path, tmp);
            return None;
        }

        root = tmp.unwrap();

        path = path[pos + 1..].to_string();

        if path.len() == 0 {
            break;
        }
    }

    Some(root)
}

pub fn ls(root_path: &str) {

    let root = match resolve_path(root_path) {
        Some(root) => root,
        None => return,
    };

    println!("     Size | Last Modified       | Name");

    if let Err(err) = fs::iterate_dir(get_volume(), &root, |entry| {
        println!("{}", entry);
    }) {
        println!("{:?}", err);
    }
}

pub fn cat(root_path: &str, file: &str) {

    let root = match resolve_path(root_path) {
        Some(root) => root,
        None => return,
    };

    let file = fs::open_file(get_volume(), &root, file, file::Mode::ReadOnly);

    if file.is_err() {
        warn!("ERROR: {:?}", file.unwrap_err());
        return;
    }

    let file = file.unwrap();

    let data = fs::read(get_volume(), &file);

    if data.is_err() {
        warn!("ERROR: {:?}", data.unwrap_err());
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
