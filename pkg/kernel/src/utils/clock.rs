use super::uefi;
use boot::BootInfo;
use chrono::naive::*;
use chrono::Duration;
use core::hint::spin_loop;

pub fn init(boot_info: &'static BootInfo) {
    if uefi::get_uefi_runtime().is_none() {
        uefi::init(boot_info);
    }
}

pub fn now() -> NaiveDateTime {
    let time = uefi::get_uefi_runtime_for_sure().get_time();

    NaiveDate::from_ymd(time.year() as i32, time.month() as u32, time.day() as u32).and_hms_nano(
        time.hour() as u32,
        time.minute() as u32,
        time.second() as u32,
        time.nanosecond(),
    )
}

pub fn spin_wait_until(time: &NaiveDateTime) {
    while &now() < time {
        spin_loop();
    }
}

pub fn spin_wait_for_ns(ns: usize) {
    spin_wait_until(&(now() + Duration::nanoseconds(ns as i64)))
}
