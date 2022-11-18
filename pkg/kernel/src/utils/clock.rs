use super::uefi;
use boot::BootInfo;
use chrono::naive::*;

pub fn init(boot_info: &'static BootInfo) {
    if uefi::get_uefi_runtime().is_none() {
        uefi::init(boot_info);
    }
}

pub fn now() -> NaiveDateTime {
    let time = uefi::get_uefi_runtime_for_sure().get_time();
    NaiveDate::from_ymd_opt(time.year() as i32, time.month() as u32, time.day() as u32)
        .unwrap_or_default()
        .and_hms_nano_opt(
            time.hour() as u32,
            time.minute() as u32,
            time.second() as u32,
            time.nanosecond(),
        )
        .unwrap_or_default()
}
