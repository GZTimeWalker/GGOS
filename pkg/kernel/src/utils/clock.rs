use chrono::{naive::*, DateTime};

pub fn now() -> NaiveDateTime {
    let time = match uefi::runtime::get_time() {
        Ok(time) => time,
        Err(_) => return DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
    };

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
