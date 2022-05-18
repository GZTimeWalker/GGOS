use crate::*;

pub fn sleep(millisecs: i64) {
    let start = sys_time();
    let dur = Duration::milliseconds(millisecs);
    let mut current = start;
    while current - start < dur {
        current = sys_time();
    }
}
