//! ATA Device Driver
//!
//! reference: https://wiki.osdev.org/IDE
//! reference: https://wiki.osdev.org/ATA_PIO_Mode
//! reference: https://github.com/xfoxfu/rust-xos/blob/main/kernel/src/drivers/ide.rs

use alloc::string::String;
use alloc::vec;
use alloc::{boxed::Box, vec::Vec};
use bit_field::BitField;
use core::hint::spin_loop;
use spin::Mutex;
use x86_64::instructions::port::*;

lazy_static! {
    static ref BUSES: Vec<Mutex<Bus>> = {
        let buses = vec![
            Mutex::new(Bus::new(0, 14, 0x1F0, 0x3F6)),
            Mutex::new(Bus::new(1, 15, 0x170, 0x376)),
        ];

        info!("Initialized ATA Buses.");

        buses
    };
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8,
    irq: u8,
    io_base: u16,
    ctrl_base: u16,
    data: Port<u16>,
    error: PortReadOnly<u8>,
    features: PortWriteOnly<u8>,
    sector_count: Port<u8>,
    sector_number: Port<u8>,
    cylinder_low: Port<u8>,
    cylinder_high: Port<u8>,
    drive: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
    alternate_status: PortReadOnly<u8>,
    control: PortWriteOnly<u8>,
    drive_blockess: PortReadOnly<u8>,
}

impl Bus {
    pub fn new(id: u8, irq: u8, io_base: u16, ctrl_base: u16) -> Self {
        Self {
            id,
            irq,
            io_base,
            ctrl_base,
            data: Port::<u16>::new(io_base),
            error: PortReadOnly::<u8>::new(io_base + 1),
            features: PortWriteOnly::<u8>::new(io_base + 1),
            sector_count: Port::<u8>::new(io_base + 2),
            sector_number: Port::<u8>::new(io_base + 3),
            cylinder_low: Port::<u8>::new(io_base + 4),
            cylinder_high: Port::<u8>::new(io_base + 5),
            drive: Port::<u8>::new(io_base + 6),
            status: PortReadOnly::new(io_base + 7),
            command: PortWriteOnly::new(io_base + 7),

            alternate_status: PortReadOnly::new(ctrl_base),
            control: PortWriteOnly::new(ctrl_base),
            drive_blockess: PortReadOnly::new(ctrl_base + 1),
        }
    }

    fn check_floating_bus(&mut self) -> Result<(), ()> {
        match self.status() {
            0xFF | 0x7F => Err(()),
            _ => Ok(()),
        }
    }
    fn status(&mut self) -> u8 {
        unsafe { self.alternate_status.read() }
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.data.read() }
    }

    fn write_data(&mut self, data: u16) {
        unsafe { self.data.write(data) }
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    fn poll(&mut self, bit: Status, val: bool) -> Result<(), ()> {
        while self.status().get_bit(bit as usize) != val {
            spin_loop();
        }
        Ok(())
    }

    fn select_drive(&mut self, drive: u8) -> Result<(), ()> {
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        unsafe {
            // Bit 4 => DEV
            // Bit 5 => 1
            // Bit 7 => 1
            self.drive.write(0xA0 | (drive << 4))
        }
        // trace!("Selected drive {}, waiting...", drive);
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        Ok(())
    }

    fn write_command_params(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);
        unsafe {
            self.sector_count.write(1);
            self.sector_number.write(bytes[0]);
            self.cylinder_low.write(bytes[1]);
            self.cylinder_high.write(bytes[2]);
            self.drive.write(bytes[3]);
        }
        // trace!("Wrote command parameters: {:016b}", block);
        Ok(())
    }

    fn setup_pio(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        self.select_drive(drive)?;
        self.write_command_params(drive, block)?;
        Ok(())
    }

    fn clear_interrupt(&mut self) -> u8 {
        unsafe { self.status.read() }
    }

    fn write_command(&mut self, cmd: ATACommand) -> Result<(), ()> {
        unsafe { self.command.write(cmd as u8) }
        // trace!("Wrote command {:?}", cmd);
        self.status(); // Ignore results of first read
        self.clear_interrupt();
        if self.status() == 0 {
            // Drive does not exist
            return Err(());
        }
        if self.is_error() {
            debug!("ATA {:?} command error", cmd);
            self.debug();
            return Err(());
        }
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, true)?;
        Ok(())
    }

    fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) -> Result<(), ()> {
        debug_assert!(buf.len() == fs::Block512::size());

        self.setup_pio(drive, block)?;
        self.write_command(ATACommand::ReadSectors)?;
        for chunk in buf.chunks_mut(2) {
            let data = self.read_data().to_le_bytes();
            chunk.clone_from_slice(&data);
        }
        if self.is_error() {
            debug!("ATA read: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    fn write(&mut self, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
        debug_assert!(buf.len() == fs::Block512::size());

        self.setup_pio(drive, block)?;
        self.write_command(ATACommand::WriteSectors)?;
        for chunk in buf.chunks(2) {
            let data = u16::from_le_bytes(chunk.try_into().unwrap());
            self.write_data(data);
        }
        if self.is_error() {
            debug!("ATA write: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    fn cylinder_low(&mut self) -> u8 {
        unsafe { self.cylinder_low.read() }
    }

    fn cylinder_high(&mut self) -> u8 {
        unsafe { self.cylinder_high.read() }
    }

    fn identify_drive(&mut self, drive: u8) -> Result<IdentifyResponse, ()> {
        if self.check_floating_bus().is_err() {
            return Ok(IdentifyResponse::None);
        }
        info!("Identifying drive {}", drive);
        self.setup_pio(drive, 0)?;
        if self.write_command(ATACommand::Identify).is_err() {
            if self.status() == 0 {
                return Ok(IdentifyResponse::None);
            } else {
                return Err(());
            }
        }
        match (self.cylinder_low(), self.cylinder_high()) {
            (0x00, 0x00) => Ok(IdentifyResponse::Ata(Box::new(
                [(); 256].map(|_| self.read_data()),
            ))),
            (0x14, 0xEB) => Ok(IdentifyResponse::Atapi),
            (0x3C, 0x3C) => Ok(IdentifyResponse::Sata),
            (_, _) => Err(()),
        }
    }

    fn debug(&mut self) {
        unsafe {
            debug!(
                "ATA status register: 0b{:08b} <BSY|DRDY|#|#|DRQ|#|#|ERR>",
                self.alternate_status.read()
            );
            debug!(
                "ATA error register : 0b{:08b} <#|#|#|#|#|ABRT|#|#>",
                self.error.read()
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ATACommand {
    ReadSectors = 0x20,
    WriteSectors = 0x30,
    Identify = 0xEC,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms, dead_code)]
enum Status {
    #[doc = "Indicates an error occurred. Send a new command to clear it (or nuke it with a Software Reset)."]
    ERR = 0,
    #[doc = "Index. Always set to zero."]
    IDX = 1,
    #[doc = "Corrected data. Always set to zero."]
    CORR = 2,
    #[doc = "Set when the drive has PIO data to transfer, or is ready to accept PIO data."]
    DRQ = 3,
    #[doc = "Overlapped Mode Service Request."]
    SRV = 4,
    #[doc = "Drive Fault Error (does not set ERR)."]
    DF = 5,
    #[doc = "Bit is clear when drive is spun down, or after an error. Set otherwise."]
    RDY = 6,
    #[doc = "Indicates the drive is preparing to send/receive data (wait for it to clear). In case of 'hang' (it never clears), do a software reset."]
    BSY = 7,
}

#[derive(Clone)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    blocks: u32,
    model: String,
    serial: String,
}

enum IdentifyResponse {
    Ata(Box<[u16; 256]>),
    Atapi,
    Sata,
    None,
}

impl Drive {
    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        trace!("Opening drive {}@{}...", bus, dsk);
        if let Ok(IdentifyResponse::Ata(res)) = BUSES[bus as usize].lock().identify_drive(dsk) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);
            let drive = Self {
                bus,
                dsk,
                model,
                serial,
                blocks,
            };
            info!("Drive {} opened", drive);
            Some(drive)
        } else {
            None
        }
    }

    pub const fn block_size(&self) -> usize {
        fs::Block512::size()
    }

    fn humanized_size(&self) -> (usize, &'static str) {
        let size = self.block_size();
        let count = self.block_count().unwrap();
        let bytes = size * count;
        if bytes >> 20 < 1024 {
            (bytes >> 20, "MiB")
        } else if bytes >> 30 < 1024 {
            (bytes >> 30, "GiB")
        } else {
            (bytes >> 40, "TiB")
        }
    }
}

impl core::fmt::Display for Drive {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

use fs::{Block512, BlockDevice};

impl BlockDevice<Block512> for Drive {
    fn block_count(&self) -> fs::Result<usize> {
        Ok(self.blocks as usize)
    }

    fn read_block(&self, offset: usize, block: &mut Block512) -> fs::Result<()> {
        BUSES[self.bus as usize]
            .lock()
            .read(self.dsk, offset as u32, block.as_mut_u8_slice())
            .map_err(|_| fs::DeviceError::ReadError.into())
    }

    fn write_block(&self, offset: usize, block: &Block512) -> fs::Result<()> {
        BUSES[self.bus as usize]
            .lock()
            .write(self.dsk, offset as u32, block.as_u8_slice())
            .map_err(|_| fs::DeviceError::WriteError.into())
    }
}
