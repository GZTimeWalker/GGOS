//! APIC (Advanced Programmable Interrupt Controller)
//!
//! For x86 kernel multi-core support.
//!
//! Migrate from:
//! * [Redox](https://github.com/redox-os/kernel/blob/master/src/arch/x86_64/device/local_apic.rs)
//! * [sv6](https://github.com/aclements/sv6/blob/master/kernel/xapic.cc)
//!
//! Reference: [OSDev Wiki](https://wiki.osdev.org/APIC)
#![allow(dead_code)]

pub use ioapic::{IoApic, IOAPIC_ADDR};
pub use xapic::{XApic, LAPIC_ADDR};

mod ioapic;
mod xapic;

type Tid = u8;

pub trait LocalApic {
    /// If this type APIC is supported
    fn support() -> bool;

    /// Initialize the LAPIC for the current CPU
    fn cpu_init(&mut self);

    /// Return this CPU's LAPIC ID
    fn id(&self) -> u32;

    fn version(&self) -> u32;

    /// Interrupt Command Register
    fn icr(&self) -> u64;

    fn set_icr(&mut self, value: u64);

    /// Acknowledge interrupt on the current CPU
    fn eoi(&mut self);

    /// Send an IPI to a remote CPU
    fn send_ipi(&mut self, apic_id: u8, int_id: u8) {
        self.set_icr((apic_id as u64) << 56 | int_id as u64);
    }

    // Start an AP
    // unsafe fn start_ap(&mut self, apic_id: Tid, addr: u32);
}
