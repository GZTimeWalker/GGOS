use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;

pub struct XApic {
    addr: usize,
}

impl XApic {
    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe { read_volatile((self.addr + reg as usize) as *const u32) }
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        unsafe {
            write_volatile((self.addr + reg as usize) as *mut u32, value);
            self.read(0x20);
        } // wait for write to finish, by reading
    }
}

impl XApic {
    pub unsafe fn new(addr: usize) -> Self {
        XApic { addr }
    }
}

impl LocalApic for XApic {
    fn support() -> bool {
        CpuId::new().get_feature_info().unwrap().has_apic()
    }

    fn cpu_init(&mut self) {
        unsafe {
            // Enable local APIC; set spurious interrupt vector.
            self.write(SVR, ENABLE | (T_IRQ0 + IRQ_SPURIOUS));

            // The timer repeatedly counts down at bus frequency
            // from lapic[TICR] and then issues an interrupt.
            // If xv6 cared more about precise timekeeping,
            // TICR would be calibrated using an external time source.
            self.write(TDCR, X1);
            self.write(TIMER, PERIODIC | (T_IRQ0 + IRQ_TIMER));
            self.write(TICR, 10000000);

            // Disable logical interrupt lines.
            self.write(LINT0, MASKED);
            self.write(LINT1, MASKED);

            // Disable performance counter overflow interrupts
            // on machines that provide that interrupt entry.
            if (self.read(VER) >> 16 & 0xFF) >= 4 {
                self.write(PCINT, MASKED);
            }

            // Map error interrupt to IRQ_ERROR.
            self.write(ERROR, T_IRQ0 + IRQ_ERROR);

            // Clear error status register (requires back-to-back writes).
            self.write(ESR, 0);
            self.write(ESR, 0);

            // Ack any outstanding interrupts.
            self.write(EOI, 0);

            // Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(ICRHI, 0);
            self.write(ICRLO, BCAST | INIT | LEVEL);
            while self.read(ICRLO) & DELIVS != 0 {}

            // Enable interrupts on the APIC (but not on the processor).
            self.write(TPR, 0);
        }
    }

    fn id(&self) -> u32 {
        unsafe { self.read(ID) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(VER) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(ICRHI) as u64) << 32 | self.read(ICRLO) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(ICRLO).get_bit(12) {}
            self.write(ICRHI, (value >> 32) as u32);
            self.write(ICRLO, value as u32);
            while self.read(ICRLO).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(EOI, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}

fn microdelay(us: u64) {
    use x86::time::rdtsc;
    let start = unsafe { rdtsc() };
    let freq = 3_000_000_000u64; // 3GHz
    let end = start + freq / 1_000_000 * us;
    while unsafe { rdtsc() } < end {}
}

pub const LAPIC_ADDR: usize = 0xfee00000;

const CMOS_PORT: u16 = 0x70;
const CMOS_RETURN: u16 = 0x71;
const ID: u32 = 0x0020; // ID
const VER: u32 = 0x0030; // Version
const TPR: u32 = 0x0080; // Task Priority
const EOI: u32 = 0x00B0; // EOI
const SVR: u32 = 0x00F0; // Spurious Interrupt Vector
const ENABLE: u32 = 0x00000100; // Unit Enable
const ESR: u32 = 0x0280; // Error Status
const ICRLO: u32 = 0x0300; // Interrupt Command
const INIT: u32 = 0x00000500; // INIT/RESET
const STARTUP: u32 = 0x00000600; // Startup IPI
const DELIVS: u32 = 0x00001000; // Delivery status
const ASSERT: u32 = 0x00004000; // Assert interrupt (vs deassert)
const DEASSERT: u32 = 0x00000000;
const LEVEL: u32 = 0x00008000; // Level triggered
const BCAST: u32 = 0x00080000; // Send to all APICs, including self.
const BUSY: u32 = 0x00001000;
const FIXED: u32 = 0x00000000;
const ICRHI: u32 = 0x0310; // Interrupt Command [63:32]
const TIMER: u32 = 0x0320; // Local Vector Table 0 (TIMER)
const X1: u32 = 0x0000000B; // divide counts by 1
const PERIODIC: u32 = 0x00020000; // Periodic
const PCINT: u32 = 0x0340; // Performance Counter LVT
const LINT0: u32 = 0x0350; // Local Vector Table 1 (LINT0)
const LINT1: u32 = 0x0360; // Local Vector Table 2 (LINT1)
const ERROR: u32 = 0x0370; // Local Vector Table 3 (ERROR)
const MASKED: u32 = 0x00010000; // Interrupt masked
const TICR: u32 = 0x0380; // Timer Initial Count
const TCCR: u32 = 0x0390; // Timer Current Count
const TDCR: u32 = 0x03E0; // Timer Divide Configuration

const T_IRQ0: u32 = 32; // IRQ 0 corresponds to int T_IRQ
const IRQ_TIMER: u32 = 0;
const IRQ_KBD: u32 = 1;
const IRQ_COM1: u32 = 4;
const IRQ_IDE: u32 = 14;
const IRQ_ERROR: u32 = 19;
const IRQ_SPURIOUS: u32 = 31;
