use super::ata::*;
use super::cache::*;
use alloc::boxed::Box;
use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

static CACHE: spin::Once<LruSharedInner> = spin::Once::new();

pub fn cache_usage() -> (usize, usize) {
    let cache = CACHE.get().unwrap().lock();
    (cache.len(), cache.cap().into())
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    let lru = LruCacheImpl::new();

    CACHE.call_once(|| lru.inner());

    let cache_layer = ATACachedDevice::new(part, lru);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(cache_layer)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let iter = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };

    println!("  Size | Last Modified       | Name");

    for meta in iter {
        let (size, unit) = crate::humanized_size_short(meta.len as u64);
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
