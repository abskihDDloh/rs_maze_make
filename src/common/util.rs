pub fn get_now_unix_time() -> u64 {
    const UNIX_EPOCH: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH;
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn get_end_time_and_elapsed_time(start_time: u64) -> (u64, u64) {
    let end_time = get_now_unix_time();
    let elapsed_time = end_time - start_time;
    (end_time, elapsed_time)
}