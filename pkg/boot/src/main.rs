//! Simple ELF OS Loader on UEFI @ 2022.03.23
//!
//! 1. Load config from "\EFI\Boot\rboot.conf"
//! 2. Load kernel ELF file
//! 3. Map ELF segments to virtual memory
//! 4. Map kernel stack and all physical memory
//! 5. Startup all processors
//! 6. Exit boot and jump to ELF entry

#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::arch::asm;
use ggos_boot::{BootInfo, GraphicInfo, KernelPages};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::*;
use uefi::table::cfg::ACPI2_GUID;
use x86_64::registers::control::*;
use x86_64::registers::model_specific::EferFlags;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::*;
use x86_64::{PhysAddr, VirtAddr};
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

    let acpi2_addr = system_table
        .config_table()
        .iter()
        .find(|entry| entry.guid == ACPI2_GUID)
        .expect("failed to find ACPI 2 RSDP")
        .address;
    info!("ACPI2: {:?}", acpi2_addr);

    let elf = {
        let mut file = open_file(bs, config.kernel_path);
        let buf = load_file(bs, &mut file);
        ElfFile::new(buf).expect("failed to parse ELF")
    };
    unsafe {
        ENTRY = elf.header.pt2.entry_point() as usize;
    }

    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap_iter = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map")
        .1;

    let max_phys_addr = mmap_iter
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    let mut page_table = current_page_table();
    // root page table is readonly
    // disable write protect
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

    // FIXME: multi-core
    //  All application processors will be shutdown after ExitBootService.
    //  Disable now.
    // start_aps(bs);

    // for i in 0..5 {
    //     info!("Waiting for next stage... {}", 5 - i);
    //     bs.stall(100_000);
    // }

    info!("Exiting boot services...");

    let (rt, mmap_iter) = system_table
        .exit_boot_services(image, mmap_storage)
        .expect("Failed to exit boot services");
    // NOTE: alloc & log can no longer be used

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap_iter.copied().collect(),
        kernel_pages: get_page_usage(&elf),
        physical_memory_offset: config.physical_memory_offset,
        graphic_info,
        system_table: rt,
    };

    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 1;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
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

/// Get current page table from CR3
fn current_page_table() -> OffsetPageTable<'static> {
    let p4_table_addr = Cr3::read().0.start_address().as_u64();
    let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
    unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
}

/// Use `BootServices::allocate_pages()` as frame allocator
struct UEFIFrameAllocator<'a>(&'a BootServices);

unsafe impl FrameAllocator<Size4KiB> for UEFIFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .expect("Failed to allocate frame");
        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        Some(frame)
    }
}

/// Open file at `path`
fn open_file(bs: &BootServices, path: &str) -> RegularFile {
    info!("Opening file: {}", path);

    let mut buf = [0; 64];

    let cstr_path = uefi::CStr16::from_str_with_buf(path, &mut buf).unwrap();

    let handle = bs
        .get_handle_for_protocol::<SimpleFileSystem>()
        .expect("Failed to get handle for SimpleFileSystem");

    let fs = bs
        .open_protocol_exclusive::<SimpleFileSystem>(handle)
        .expect("Failed to get FileSystem");
    let mut fs = fs;

    let mut root = fs.open_volume().expect("Failed to open volume");
    let handle = root
        .open(cstr_path, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open file");

    match handle.into_type().expect("Failed to into_type") {
        FileType::Regular(regular) => regular,
        _ => panic!("Invalid file type"),
    }
}

/// Load file to new allocated pages
fn load_file(bs: &BootServices, file: &mut RegularFile) -> &'static mut [u8] {
    info!("Loading file to memory");
    let mut info_buf = [0u8; 0x100];
    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("Failed to get file info");
    let pages = info.file_size() as usize / 0x1000 + 1;
    let mem_start = bs
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages");
    let buf = unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };
    let len = file.read(buf).expect("Failed to read file");
    info!("File size={}", len);
    &mut buf[..len]
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

/// The entry point of kernel, set by BSP.
static mut ENTRY: usize = 0;

/// Jump to ELF entry according to global variable `ENTRY`
unsafe fn jump_to_entry(bootinfo: *const BootInfo, stacktop: u64) -> ! {
    asm!("mov rsp, {}; call {}", in(reg) stacktop, in(reg) ENTRY, in("rdi") bootinfo);
    loop {}
}
