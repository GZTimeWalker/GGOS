mod apic;
mod consts;
mod handlers;
mod keyboard;
mod serial;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            handlers::reg_idt(&mut idt);
            keyboard::reg_idt(&mut idt);
            serial::reg_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();
    debug!("XApic support = {}.", apic::XApic::support());
    let mut lapic = unsafe { XApic::new(crate::memory::physical_to_virtual(LAPIC_ADDR)) };
    lapic.cpu_init();
    keyboard::init();
    serial::init();

    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8) {
    let mut ioapic =
        unsafe { IoApic::new(crate::memory::physical_to_virtual(IOAPIC_ADDR as u64)) };
    ioapic.enable(irq, 0);
}

#[inline(always)]
pub fn ack(_irq: u8) {
    let mut lapic = unsafe { XApic::new(crate::memory::physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
