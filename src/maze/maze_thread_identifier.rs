use std::{
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::NaiveDateTime;
use rand::Rng;
use sea_orm::prelude::DateTimeUtc;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct MazeThreadIdentifier {
    /// スレッドID（生成元スレッドの識別）
    tid_id: thread::ThreadId,
    /// UNIX時刻（ミリ秒精度）
    unix_time: i64,
}

impl MazeThreadIdentifier {
    pub fn new() -> Self {
        // 5-10msのランダムスリープを入れて、UNIX時間の重複を回避する
        use std::time::Duration;
        let mut rng = rand::rng();
        let sleep_time = rng.random_range(5..=10);
        thread::sleep(Duration::from_millis(sleep_time));
        let tid_id: thread::ThreadId = thread::current().id();
        // SystemTimeを使用して取得（ミリ秒精度）
        let unix_time: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;
        MazeThreadIdentifier { tid_id, unix_time }
    }

    pub fn thread_id(&self) -> thread::ThreadId {
        self.tid_id
    }
    pub fn thread_id_as_str(&self) -> String {
        format!("{:?}", self.thread_id())
    }
    pub fn unix_time(&self) -> i64 {
        self.unix_time
    }
    pub fn unix_time_as_date_time_utc(&self) -> Result<DateTimeUtc, Box<dyn std::error::Error>> {
        let secs = self.unix_time / 1_000;
        let millis = (self.unix_time % 1_000).abs();
        let dt_utc = DateTimeUtc::from_timestamp_millis(secs * 1_000 + millis);
        let dt_utc = dt_utc.ok_or("Failed to convert unix_time to DateTimeUtc")?;
        Ok(dt_utc)
    }
    pub fn same_thread(&self, other: &MazeThreadIdentifier) -> bool {
        self.tid_id == other.tid_id
    }
    pub fn as_str(&self) -> String {
        format!("{}_{}", self.thread_id_as_str(), self.unix_time)
    }
}

impl std::fmt::Display for MazeThreadIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_uses_current_thread_id() {
        let identifier = MazeThreadIdentifier::new();
        assert_eq!(identifier.thread_id(), thread::current().id());
    }

    #[test]
    fn same_thread_distinguishes_threads() {
        let main_identifier = MazeThreadIdentifier::new();
        assert!(main_identifier.same_thread(&main_identifier));

        let other_identifier = thread::spawn(MazeThreadIdentifier::new)
            .join()
            .expect("failed to join thread");

        assert!(!main_identifier.same_thread(&other_identifier));
        assert!(!other_identifier.same_thread(&main_identifier));
    }

    #[test]
    fn as_str_contains_id_and_timestamp() {
        let identifier = MazeThreadIdentifier::new();
        let thread_repr = identifier.thread_id_as_str();
        let timestamp = identifier.unix_time().to_string();

        let combined = identifier.as_str();

        assert!(combined.starts_with(&thread_repr));
        assert!(combined.ends_with(&timestamp));
        assert_eq!(combined, identifier.to_string());
    }

    #[test]
    fn unix_time_is_monotonic() {
        let first = MazeThreadIdentifier::new();
        let second = MazeThreadIdentifier::new();

        assert!(second.unix_time() >= first.unix_time());
    }

    #[test]
    fn unix_time_as_date_time_utc_matches_current_behavior() {
        let identifier = MazeThreadIdentifier::new();

        let dt = identifier
            .unix_time_as_date_time_utc()
            .expect("failed to convert to datetime");

        // DateTimeUtc::timestamp は秒精度。ミリ秒が下3桁として含まれていることを確認。
        let millis_component = (identifier.unix_time() % 1_000) as u32;
        assert_eq!(dt.timestamp_millis() % 1_000, millis_component as i64);
    }
}
