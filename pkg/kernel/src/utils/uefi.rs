use boot::*;
use crate::{once_mutex, guard_access_fn};

once_mutex!(UEFI_SERVICE: UefiRuntime);

pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        init_UEFI_SERVICE(UefiRuntime::new(boot_info));
    }
}

guard_access_fn! {
    pub get_uefi_runtime(UEFI_SERVICE: UefiRuntime)
}

pub struct UefiRuntime {
    runtime_service: &'static RuntimeServices
}

impl UefiRuntime {
    pub unsafe fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            runtime_service: boot_info.system_table.runtime_services()
        }
    }

    pub fn get_time(&self) -> Time {
        self.runtime_service.get_time().unwrap()
    }
}
