use super::uefi;
use boot::BootInfo;
use chrono::naive::*;
use chrono::Duration;

pub fn init(boot_info: &'static BootInfo) {
    if uefi::get_uefi_runtime().is_none() {
        uefi::init(boot_info);
    }
}

pub fn now() -> NaiveDateTime{
    let uefi_time = uefi::get_uefi_runtime_for_sure().get_time();

    NaiveDate::from_ymd(
        uefi_time.year() as i32,
        uefi_time.month() as u32,
        uefi_time.day() as u32,
    ).and_hms_nano(
        uefi_time.hour() as u32,
        uefi_time.minute() as u32,
        uefi_time.second() as u32,
        uefi_time.nanosecond(),
    )
}

pub fn spin_wait_until(time: &NaiveDateTime) {
    while &now() < time {}
}

pub fn spin_wait_for_ns(ns: usize) {
    spin_wait_until(
        &(now() + Duration::nanoseconds(ns as i64))
    )
}
