use arrayvec::{ArrayString, ArrayVec};
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::*;
use uefi::Char16;
use xmas_elf::ElfFile;

use crate::App;

/// Open root directory
pub fn open_root(bs: &BootServices) -> Directory {
    let handle = bs
        .get_handle_for_protocol::<SimpleFileSystem>()
        .expect("Failed to get handle for SimpleFileSystem");

    let fs = bs
        .open_protocol_exclusive::<SimpleFileSystem>(handle)
        .expect("Failed to get FileSystem");
    let mut fs = fs;

    fs.open_volume().expect("Failed to open volume")
}

/// Open file at `path`
pub fn open_file(bs: &BootServices, path: &str) -> RegularFile {
    let mut buf = [0; 64];
    let cstr_path = uefi::CStr16::from_str_with_buf(path, &mut buf).unwrap();

    let handle = open_root(bs)
        .open(cstr_path, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open file");

    match handle.into_type().expect("Failed to into_type") {
        FileType::Regular(regular) => regular,
        _ => panic!("Invalid file type"),
    }
}

/// Load file to new allocated pages
pub fn load_file(bs: &BootServices, file: &mut RegularFile) -> &'static mut [u8] {
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

    info!(
        "Load file \"{}\" to memory, size = {}",
        info.file_name(),
        len
    );
    &mut buf[..len]
}

/// Load apps into memory, when no fs implemented in kernel
///
/// List all file under "APP" and load them.
pub fn load_apps(bs: &BootServices) -> ArrayVec<App<'static>, 16> {
    let mut root = open_root(bs);

    let mut buf = [0; 8];
    let cstr_path = uefi::CStr16::from_str_with_buf("\\APP\\", &mut buf).unwrap();

    let mut handle = root
        .open(cstr_path, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open file")
        .into_directory()
        .expect("App directory not found");

    let mut apps = ArrayVec::new();

    let mut buffer = [0u8; 0x100];

    loop {
        let info = handle
            .read_entry(&mut buffer)
            .expect("Failed to read entry");

        match info {
            Some(info) => {
                // skip when info begin with "."
                if info.file_name().as_slice()[0] == Char16::try_from('.').unwrap() {
                    continue;
                }

                let file = handle
                    .open(info.file_name(), FileMode::Read, FileAttribute::empty())
                    .expect("Failed to open file");

                if file.is_directory().unwrap_or(true) {
                    continue;
                }

                let mut file = file.into_regular_file().unwrap();
                let buf = load_file(bs, &mut file);

                let elf = ElfFile::new(buf).expect("Failed to parse ELF file");
                let mut name = ArrayString::<16>::new();

                info.file_name().as_str_in_buf(&mut name).unwrap();

                apps.push(App { name, elf });
            }
            None => break,
        }
    }

    info!("Loaded {} apps", apps.len());

    apps
}
