#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::arch::asm;
use ggos_boot::allocator::*;
use ggos_boot::fs::*;
use ggos_boot::*;
use uefi::prelude::*;
use x86_64::registers::control::*;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::*;
use x86_64::VirtAddr;
use xmas_elf::program::ProgramHeader;
use xmas_elf::ElfFile;

mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    let bs = system_table.boot_services();
    let config = {
        let mut file = open_file(bs, CONFIG_PATH);
        let buf = load_file(bs, &mut file);
        config::Config::parse(buf)
    };

    let graphic_info = init_graphic(bs);
    info!("config: {:#x?}", config);

    let elf = {
        let mut file = open_file(bs, config.kernel_path);
        let buf = load_file(bs, &mut file);
        ElfFile::new(buf).expect("failed to parse ELF")
    };
    unsafe {
        ENTRY = elf.header.pt2.entry_point() as usize;
    }

    let apps = if config.load_apps {
        info!("Loading apps...");
        Some(load_apps(system_table.boot_services()))
    } else {
        info!("Skip loading apps");
        None
    };

    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();
    // root page table is readonly, disable write protect
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
        Efer::update(|f| f.insert(EferFlags::NO_EXECUTE_ENABLE));
    }

    elf::map_elf(&elf, &mut page_table, &mut UEFIFrameAllocator(bs)).expect("Failed to map ELF");

    let (stack_start, stack_size) = if config.kernel_stack_auto_grow > 0 {
        let stack_start = config.kernel_stack_address
            + (config.kernel_stack_size - config.kernel_stack_auto_grow) * 0x1000;
        (stack_start, config.kernel_stack_auto_grow)
    } else {
        (config.kernel_stack_address, config.kernel_stack_size)
    };

    info!(
        "Kernel init stack: [0x{:x?} -> 0x{:x?})",
        stack_start,
        stack_start + stack_size * 0x1000
    );

    elf::map_range(
        stack_start,
        stack_size,
        &mut page_table,
        &mut UEFIFrameAllocator(bs),
        false,
    )
    .expect("Failed to map stack");

    elf::map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut UEFIFrameAllocator(bs),
    );

    // recover write protect
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }

    // 5. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let (runtime, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    // NOTE: alloc & log can no longer be used

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        kernel_pages: get_page_usage(&elf),
        physical_memory_offset: config.physical_memory_offset,
        system_table: runtime,
        loaded_apps: apps,
        log_level: config.log_level,
        graphic_info,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}

/// Get current page table from CR3
fn current_page_table() -> OffsetPageTable<'static> {
    let p4_table_addr = Cr3::read().0.start_address().as_u64();
    let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
    unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
}

pub fn get_page_usage(elf: &ElfFile) -> KernelPages {
    elf.program_iter()
        .filter(|segment| segment.get_type().unwrap() == xmas_elf::program::Type::Load)
        .map(|segment| get_page_range(segment))
        .collect()
}

fn get_page_range(header: ProgramHeader) -> PageRangeInclusive {
    let virt_start_addr = VirtAddr::new(header.virtual_addr());
    let mem_size = header.mem_size();
    let start_page = Page::containing_address(virt_start_addr);
    let end_page = Page::containing_address(virt_start_addr + mem_size - 1u64);
    Page::range_inclusive(start_page, end_page)
}

/// If `resolution` is some, then set graphic mode matching the resolution.
/// Return information of the final graphic mode.
fn init_graphic(bs: &BootServices) -> GraphicInfo {
    let handle = bs
        .get_handle_for_protocol::<GraphicsOutput>()
        .expect("Failed to get GOP handle");
    let gop = bs
        .open_protocol_exclusive::<GraphicsOutput>(handle)
        .expect("Failed to get GraphicsOutput");

    let mut gop = gop;

    GraphicInfo {
        mode: gop.current_mode_info(),
        fb_addr: gop.frame_buffer().as_mut_ptr() as u64,
        fb_size: gop.frame_buffer().size() as u64,
    }
}

/// The entry point of kernel, set by BSP.
static mut ENTRY: usize = 0;

/// Jump to ELF entry according to global variable `ENTRY`
#[allow(clippy::empty_loop)]
unsafe fn jump_to_entry(bootinfo: *const BootInfo, stacktop: u64) -> ! {
    asm!("mov rsp, {}; call {}", in(reg) stacktop, in(reg) ENTRY, in("rdi") bootinfo);
    loop {}
}
