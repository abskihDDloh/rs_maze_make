use std::thread;

use rand::Rng;
use rs_maze_maker::common::{
    database::{entities::thread_list, initializer::OutsideWallConnectTypeEnum},
    util::get_now_unix_time,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MazeThreadIdentifier {
    table_thread_id: u64,
    /// スレッドID（生成元スレッドの識別）
    tid_id: thread::ThreadId,
    /// UNIX時刻（ナノ秒精度）
    unix_time: i64,
    // 参照ではなく所有する
    outside_wall_connect_type: String,
}

impl MazeThreadIdentifier {
    /// 新しいMazeThreadIdentifierを生成します。(データベースのスレッドレコードに登録するためのキー情報のみを持ちます)
    /// # 戻り値
    /// MazeThreadIdentifierのインスタンス
    pub(in crate::maze) fn new() -> Self {
        // 5-10msのランダムスリープを入れて、UNIX時間の重複を回避する
        use std::time::Duration;
        let mut rng = rand::rng();
        let sleep_time = rng.random_range(5..=10);
        thread::sleep(Duration::from_millis(sleep_time));
        let tid_id: thread::ThreadId = thread::current().id();
        // SystemTimeを使用して取得（ナノ秒精度）
        let unix_time: i64 = get_now_unix_time();
        MazeThreadIdentifier {
            table_thread_id: 0,
            tid_id,
            unix_time,
            outside_wall_connect_type: String::new(), // or "".to_string()
        }
    }

    /// データベースのスレッドレコードに登録した情報を含んだMazeThreadIdentifierを生成します。
    /// # 引数
    /// * `record_id` - データベースから取得したスレッドレコードのID
    /// * `outside_wall_connect_type` - データベースから取得した外壁接続タイプ
    /// # 戻り値
    /// MazeThreadIdentifierのインスタンス
    pub(in crate::maze) fn fill_info(
        &self,
        record_id: u64,
        outside_wall_connect_type: String,
    ) -> Self {
        MazeThreadIdentifier {
            table_thread_id: record_id,
            tid_id: self.tid_id,
            unix_time: self.unix_time,
            outside_wall_connect_type,
        }
    }
}

impl Default for MazeThreadIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

impl MazeThreadIdentifier {
    pub fn table_thread_id(&self) -> u64 {
        self.table_thread_id
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
    pub fn same_thread(&self, other: &MazeThreadIdentifier) -> bool {
        self.tid_id == other.tid_id
    }
    pub fn as_str(&self) -> String {
        format!("{}_{}", self.thread_id_as_str(), self.unix_time)
    }
    pub fn outside_wall_connect_type(&self) -> &str {
        &self.outside_wall_connect_type
    }
    pub fn outside_wall_connect_type_enum(
        &self,
    ) -> Result<OutsideWallConnectTypeEnum, strum::ParseError> {
        self.outside_wall_connect_type.parse()
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
}
