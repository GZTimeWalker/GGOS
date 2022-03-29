mod apic;
mod consts;
mod handlers;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;

pub use handlers::Registers;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            handlers::reg_idt(&mut idt);
        }
        idt
    };
}

/// 初始化中断及硬件系统；不会打开中断
pub unsafe fn init() {
    IDT.load();
    info!("XApic support = {}", apic::XApic::support());
    let mut xapic = XApic::new(crate::memory::physical_to_virtual(0xfee00000));
    xapic.cpu_init();
}

#[inline(always)]
pub fn ack(_irq: u8) {
    let mut lapic = unsafe { XApic::new(crate::memory::physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
