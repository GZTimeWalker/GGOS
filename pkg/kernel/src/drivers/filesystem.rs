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
pub enum StdIO {
    Stdin,
    Stdout,
    Stderr,
}

pub fn init() {
    info!("Opening disk device...");
    let [p0, _, _, _] = Disk::new(Drive::open(0, 0).unwrap()).volumes();

    info!("Mounting filesystem...");
    FILESYSTEM.call_once(|| FAT16Volume::new(p0));

    info!("Initialized Filesystem.");
}

pub fn resolve_path(root_path: &str) -> Option<Directory> {
    let mut path = root_path.to_owned();
    let mut root = fs::root_dir();

    while let Some(pos) = path.find('/') {
        let dir = path[..pos].to_owned();

        let tmp = fs::find_directory_entry(get_volume(), &root, dir.as_str());

        if tmp.is_err() {
            warn!("Directory not found: {}, {:?}", root_path, tmp);
            return None;
        }

        root = tmp.unwrap();

        path = path[pos + 1..].to_string();
        trace!("Resolving path: {}", path);

        if path.len() == 0 {
            break;
        }
    }

    Some(root)
}

pub fn try_get_file(path: &str, mode: file::Mode) -> Result<File, VolumeError> {
    let path = path.to_owned();
    let pos = path.rfind('/');

    if pos.is_none() {
        return Err(VolumeError::FileNotFound);
    }
    let pos = pos.unwrap();

    trace!("root: {}, filename: {}", &path[..pos + 1], &path[pos + 1..]);

    let root = resolve_path(&path[..=pos]);
    let filename = &path[pos + 1..];

    if root.is_none() {
        return Err(VolumeError::FileNotFound);
    }
    let root = root.unwrap();

    fs::open_file(get_volume(), &root, filename, mode)
}

pub fn ls(root_path: &str) {
    let root = match resolve_path(root_path) {
        Some(root) => root,
        None => return,
    };

    println!("  Size | Last Modified       | Name");

    if let Err(err) = fs::iterate_dir(get_volume(), &root, |entry| {
        println!("{}", entry);
    }) {
        println!("{:?}", err);
    }
}
