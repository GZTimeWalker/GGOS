use crate::ata::*;
use fs::*;
use spin::Mutex;

pub type Disk = fs::device::Disk<'static, MutexDrive<'static>>;
pub type Volume = fs::device::FAT16Volume<'static, MutexDrive<'static>>;

once_mutex!(DRIVE: Drive);
guard_access_fn!(get_drive(DRIVE: Drive));

pub static FILESYSTEM: spin::Once<Volume> = spin::Once::new();

pub fn fs() -> &'static Volume {
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
