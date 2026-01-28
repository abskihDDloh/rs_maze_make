use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

pub fn get_now_unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as i64
}

pub fn get_end_time_and_elapsed_time(start_time: i64) -> (i64, i64) {
    let end_time = get_now_unix_time();
    let elapsed_time = end_time - start_time;
    (end_time, elapsed_time)
}

pub fn get_late_10_percent_flag() -> bool {
    let random_value: u8 = rand::rng().random_range(0..10);
    return random_value == 0
}
