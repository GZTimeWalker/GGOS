use crate::*;

pub fn sleep(millisecs: i64) {
    let start = sys_time();
    let dur = Duration::try_milliseconds(millisecs).unwrap();
    let mut current = start;
    while current - start < dur {
        current = sys_time();
    }
}
