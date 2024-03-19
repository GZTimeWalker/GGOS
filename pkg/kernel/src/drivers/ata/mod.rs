//! ATA Device Driver
//!
//! reference: https://wiki.osdev.org/IDE
//! reference: https://wiki.osdev.org/ATA_PIO_Mode
//! reference: https://github.com/xfoxfu/rust-xos/blob/main/kernel/src/drivers/ide.rs
//! reference: https://github.com/theseus-os/Theseus/blob/HEAD/kernel/ata/src/lib.rs

mod bus;
mod consts;

use alloc::{boxed::Box, string::String};
use bus::AtaBus;
use consts::AtaDeviceType;
use spin::Mutex;

lazy_static! {
    pub static ref BUSES: [Mutex<AtaBus>; 2] = {
        let buses = [
            Mutex::new(AtaBus::new(0, 14, 0x1F0, 0x3F6)),
            Mutex::new(AtaBus::new(1, 15, 0x170, 0x376)),
        ];

        info!("Initialized ATA Buses.");

        buses
    };
}

#[derive(Clone)]
pub struct AtaDrive {
    pub bus: u8,
    pub drive: u8,
    blocks: u32,
    model: Box<str>,
    serial: Box<str>,
}

impl AtaDrive {
    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        trace!("Opening drive {}@{}...", bus, dsk);
        if let Ok(AtaDeviceType::Pata(res)) = BUSES[bus as usize].lock().identify_drive(dsk) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);
            let drive = Self {
                bus,
                drive: dsk,
                model,
                serial,
                blocks,
            };
            info!("Drive {} opened", drive);
            Some(drive)
        } else {
            warn!("Drive {}@{} is not a PATA drive", bus, dsk);
            None
        }
    }

    fn humanized_size(&self) -> (f32, &'static str) {
        let size = self.block_size();
        let count = self.block_count().unwrap();
        let bytes = size * count;

        crate::humanized_size(bytes as u64)
    }
}

impl core::fmt::Display for AtaDrive {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

use storage::{Block512, BlockDevice};

impl BlockDevice<Block512> for AtaDrive {
    fn block_count(&self) -> storage::Result<usize> {
        Ok(self.blocks as usize)
    }

    fn read_block(&self, offset: usize, block: &mut Block512) -> storage::Result<()> {
        BUSES[self.bus as usize]
            .lock()
            .read_pio(self.drive, offset as u32, block.as_mut())
    }

    fn write_block(&self, offset: usize, block: &Block512) -> storage::Result<()> {
        BUSES[self.bus as usize]
            .lock()
            .write_pio(self.drive, offset as u32, block.as_ref())
    }
}
