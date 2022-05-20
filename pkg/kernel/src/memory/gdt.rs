use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const SYSCALL_IST_INDEX: u16 = 1;
pub const PAGE_FAULT_IST_INDEX: u16 = 2;
pub const CONTEXT_SWITCH_IST_INDEX: u16 = 0;

pub const IST_SIZES: [usize; 3] = [0x1000, 0x4000, 0x1000];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[0];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            info!("Double Fault IST: 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
            stack_end
        };
        tss.interrupt_stack_table[SYSCALL_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[1];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            info!("Syscall IST     : 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
            stack_end
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[2];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            info!("Page Fault IST  : 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;
    use x86_64::PrivilegeLevel;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        SS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        ES::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        FS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        GS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        load_tss(GDT.1.tss_selector);
    }

    let mut size = 0;

    for &s in IST_SIZES.iter() {
        size += s;
    }

    info!("Kernel IST Size : {} KiB", size / 1024);

    info!("GDT Initialized.");
}
