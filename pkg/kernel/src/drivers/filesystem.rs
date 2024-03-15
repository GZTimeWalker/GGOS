use crate::ata::*;
use alloc::boxed::Box;
use chrono::DateTime;
use fs::{FileSystem, Mount};

pub type Disk = fs::mbr::Disk<Drive>;
pub type Volume = fs::mbr::Volume<Drive>;
pub type Fat16 = fs::fat16::Fat16<Volume>;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

#[derive(Debug, Clone)]
pub enum StdIO {
    Stdin,
    Stdout,
    Stderr,
}

pub fn init() {
    info!("Opening disk device...");
    let disk = Disk::new(Drive::open(0, 0).unwrap());

    // QEMU's default disk image has a single partition
    let [p0, _, _, _] = disk.volumes().unwrap();

    info!("Mounting filesystem...");
    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(p0)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let iter = get_rootfs().read_dir(root_path);

    if let Err(err) = iter {
        warn!("{:?}", err);
        return;
    }

    let iter = iter.unwrap();

    println!("  Size | Last Modified       | Name");

    for meta in iter {
        let (size, unit) = fs::humanized_size(meta.len);
        println!(
            "{:>5.*}{} | {} | {}{}",
            1,
            size,
            unit,
            meta.modified
                .map(|t| t.format("%Y/%m/%d %H:%M:%S"))
                .unwrap_or(
                    DateTime::from_timestamp_millis(0)
                        .unwrap()
                        .format("%Y/%m/%d %H:%M:%S")
                ),
            meta.name,
            if meta.is_dir() { "/" } else { "" }
        );
    }
}
