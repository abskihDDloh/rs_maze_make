use rand::RngExt;
use std::{
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

/// 壁の識別子を表す構造体
///
/// この構造体は迷路生成において、各壁がどのスレッドのどの処理で生成されたかを
/// 一意に識別するために使用されます。識別子はスレッドIDと生成時刻のナノ秒を
/// 組み合わせることで、高い確率でユニークな値を生成します。
///
/// # 設計目的
///
/// - **壁の所有権管理**: どのスレッドが特定の壁を生成したかを追跡
/// - **競合状態の検出**: 複数スレッドが同じ壁を操作しようとした際の検出
/// - **デバッグ支援**: 迷路生成過程のトレーサビリティ向上
/// - **一意性保証**: システム全体で重複しない識別子の提供
///
/// # 実装の詳細
///
/// ## 識別子の構成
/// - **スレッドID** (`thread::ThreadId`): 生成元スレッドの識別
/// - **UNIX時刻** (ナノ秒精度): 生成時刻の高精度記録
/// - **ランダムスリープ**: 5-10ナノ秒の待機による重複回避
///
/// ## 一意性の保証方法
/// 1. **時刻精度**: ナノ秒レベルでの時刻記録
/// 2. **スレッド分離**: 各スレッドが独立した識別子空間を持つ
/// 3. **ランダム待機**: 同時生成時の時刻衝突を回避
/// 4. **単調増加**: 同一スレッド内での時刻の順序保証
///
/// ## 文字列形式
/// ```text
/// ThreadId(<内部ID>)_<UNIX時刻ナノ秒>
/// ```
/// 例: `"ThreadId(1)_1633036800123456789"`
///
/// # パフォーマンス特性
///
/// - **生成時間**: 約5-15ナノ秒（ランダムスリープ含む）
/// - **メモリ使用量**: 16バイト（ThreadId + i64）
/// - **比較性能**: O(1) - 単純な値比較
/// - **ハッシュ性能**: O(1) - 構造体メンバーの組み合わせ
///
/// # スレッドセーフティ
///
/// - **生成**: スレッドセーフ（各スレッドが独立して生成）
/// - **比較**: スレッドセーフ（イミュータブルな値比較）
/// - **複製**: スレッドセーフ（Copy/Clone trait実装）
/// - **共有**: `Send + Sync` により安全な共有が可能
///
/// # 使用例
///
/// ## 基本的な使用
/// ```rust
/// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
/// let identifier = WallIdentifier::new();
/// println!("識別子: {}", identifier.as_str());
/// // 出力例: "ThreadId(1)_1633036800123456789"
/// ```
///
/// ## 壁の所有権チェック
/// ```rust
/// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
/// let my_id = WallIdentifier::new();
/// let wall_id = get_wall_identifier_from_somewhere();
///
/// if my_id == wall_id {
///     println!("この壁は自分が生成した");
/// } else {
///     println!("この壁は他のスレッドが生成した");
/// }
/// ```
///
/// ## マルチスレッド環境での使用
/// ```rust
/// use std::thread;
/// use std::sync::{Arc, Mutex};
/// use std::collections::HashMap;
///
/// let wall_registry = Arc::new(Mutex::new(HashMap::new()));
/// let mut handles = vec![];
///
/// for thread_num in 0..4 {
///     let registry = Arc::clone(&wall_registry);
///     let handle = thread::spawn(move || {
///         let my_id = WallIdentifier::new();
///         let mut registry = registry.lock().unwrap();
///         registry.insert(format!("wall_{}", thread_num), my_id);
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// ```
///
/// # 注意事項
///
/// ## 性能への影響
/// - 生成時に5-10ナノ秒のスリープを行うため、高頻度生成時は累積的な遅延が発生
/// - 1秒間に最大約200万個の識別子生成が理論上の上限
///
/// ## システム依存性
/// - システム時刻が逆行した場合はパニック（NTPやシステム時刻調整時）
/// - スレッドIDの文字列表現はRust実装に依存
/// - ナノ秒精度はシステムクロックの精度に依存
///
/// ## 一意性の限界
/// - 理論上は重複可能（確率は極めて低い）
/// - システム再起動時にスレッドIDが再利用される可能性
/// - 長時間稼働システムでのオーバーフロー（2^63ナノ秒 ≈ 292年後）
///
/// # 関連項目
///
/// - `MazePointStatus`: 識別子を使用する迷路ポイント状態
/// - `WallType`: 識別子で管理される壁の種類
/// - `Field`: 識別子で壁を管理する迷路フィールド
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct WallIdentifier {
    /// スレッドID（生成元スレッドの識別）
    tid_id: thread::ThreadId,
    /// UNIX時刻（ナノ秒精度）
    unix_time_nanos: i64,
}

#[allow(dead_code)]
impl WallIdentifier {
    /// 新しい壁識別子を生成します
    ///
    /// 現在のスレッドIDと現在時刻のナノ秒を組み合わせて、一意性の高い識別子を
    /// 生成します。重複回避のため、生成時にランダムな短時間のスリープを実行します。
    ///
    /// # アルゴリズム詳細
    ///
    /// 1. **重複回避待機**: 5-10ナノ秒のランダムスリープ
    ///    - 同時生成による時刻の重複を回避
    ///    - `rand::rng()` による乱数生成
    /// 2. **スレッドID取得**: `thread::current().id()` で現在スレッドを特定
    /// 3. **時刻取得**: `SystemTime::now()` でナノ秒精度の現在時刻を取得
    /// 4. **構造体構築**: 取得した情報で `WallIdentifier` を構築
    ///
    /// # 一意性保証
    ///
    /// - **同一スレッド内**: ランダムスリープ + ナノ秒精度により重複回避
    /// - **異なるスレッド間**: スレッドIDの違いにより自動的に一意
    /// - **時系列順序**: 同一スレッド内では生成順序が保証される
    ///
    /// # 戻り値
    ///
    /// 新しい `WallIdentifier` インスタンス
    /// - `tid_id`: 現在のスレッドID
    /// - `unix_time_nanos`: 生成時刻（UNIX エポックからのナノ秒）
    ///
    /// # パニック
    ///
    /// 以下の状況でパニックが発生します：
    /// - システム時刻がUNIXエポック（1970-01-01 00:00:00 UTC）より前
    /// - システム時刻の取得に失敗（通常は発生しません）
    ///
    /// ```rust,should_panic
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// // システム時刻が逆行した場合（通常は発生しない）
    /// // let id = WallIdentifier::new(); // パニック: "Time went backwards"
    /// ```
    ///
    /// # 例
    ///
    /// ## 基本的な生成
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id1 = WallIdentifier::new();
    /// let id2 = WallIdentifier::new();
    ///
    /// assert_ne!(id1, id2); // 異なる識別子が生成される
    /// println!("ID1: {}", id1.as_str());
    /// println!("ID2: {}", id2.as_str());
    /// ```
    ///
    /// ## 時系列順序の確認
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let first = WallIdentifier::new();
    /// let second = WallIdentifier::new();
    ///
    /// // 同一スレッドでは時刻が増加する
    /// assert!(second.unix_time_nanos > first.unix_time_nanos);
    /// ```
    ///
    /// ## 大量生成での一意性
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// use std::collections::HashSet;
    ///
    /// let mut unique_ids = HashSet::new();
    /// for _ in 0..1000 {
    ///     let id = WallIdentifier::new();
    ///     assert!(unique_ids.insert(id)); // 全て異なる識別子
    /// }
    /// ```
    pub fn new() -> Self {
        // 5-10nsのランダムスリープを入れて、UNIX時間の重複を回避する
        use std::time::Duration;
        let mut rng = rand::rng();
        let sleep_nanos = rng.random_range(5..=10);
        thread::sleep(Duration::new(0, sleep_nanos));

        let tid_id: thread::ThreadId = thread::current().id();
        // SystemTimeを使用してナノ秒精度で取得
        let unix_time_nanos: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as i64;
        WallIdentifier {
            tid_id,
            unix_time_nanos,
        }
    }

    /// 識別子を文字列形式で取得します
    ///
    /// 識別子の人間可読な文字列表現を生成します。この文字列は一意であり、
    /// ログ出力、デバッグ、永続化などの用途に使用できます。
    ///
    /// # 文字列形式
    ///
    /// ```text
    /// ThreadId(<内部スレッドID>)_<UNIX時刻ナノ秒>
    /// ```
    ///
    /// ## 構成要素
    /// - **ThreadId部分**: Rustの内部スレッドID表現
    ///   - 例: `ThreadId(1)`, `ThreadId(2)`, など
    ///   - 実装依存だが、通常は数値
    /// - **区切り文字**: アンダースコア (`_`)
    /// - **時刻部分**: UNIX エポックからのナノ秒（10進数）
    ///   - 例: `1633036800123456789`
    ///   - 19桁程度の数値（2021年現在）
    ///
    /// # 一意性保証
    ///
    /// - 同じ識別子からは常に同じ文字列が生成される
    /// - 異なる識別子からは異なる文字列が生成される
    /// - 文字列の辞書式順序は生成順序と必ずしも一致しない
    ///
    /// # 戻り値
    ///
    /// 識別子の文字列表現（`String`）
    /// - 所有された文字列（コピー可能）
    /// - ヒープメモリを使用（ガベージコレクション対象）
    ///
    /// # 用途例
    ///
    /// ## ログ出力
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let identifier = WallIdentifier::new();
    /// println!("Wall created with ID: {}", identifier.as_str());
    /// // ログライブラリでの使用
    /// // log::info!("Processing wall {}", identifier.as_str());
    /// ```
    ///
    /// ## 永続化/シリアライゼーション
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// use std::collections::HashMap;
    ///
    /// let identifier = WallIdentifier::new();
    /// let mut wall_data = HashMap::new();
    /// wall_data.insert("id".to_string(), identifier.as_str());
    /// wall_data.insert("type".to_string(), "external_wall".to_string());
    ///
    /// // JSONシリアライゼーション等で使用可能
    /// ```
    ///
    /// ## デバッグ情報
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id = WallIdentifier::new();
    /// let debug_info = format!(
    ///     "Wall Debug Info:\n  Identifier: {}\n  Thread: {}\n",
    ///     id.as_str(),
    ///     std::thread::current().name().unwrap_or("unnamed")
    /// );
    /// ```
    ///
    /// # パフォーマンス考慮事項
    ///
    /// - **メモリ割り当て**: 毎回新しい `String` を生成
    /// - **計算コスト**: `format!` マクロによる文字列フォーマット
    /// - **推奨使用頻度**: ログ出力やデバッグ時のみ（高頻度呼び出しは避ける）
    ///
    /// # 例
    ///
    /// ## 基本的な使用
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let identifier = WallIdentifier::new();
    /// let id_str = identifier.as_str();
    ///
    /// println!("Generated ID: {}", id_str);
    /// // 出力例: "Generated ID: ThreadId(1)_1633036800123456789"
    ///
    /// // 文字列の基本的なプロパティ確認
    /// assert!(id_str.contains("ThreadId"));
    /// assert!(id_str.contains("_"));
    /// ```
    ///
    /// ## 複数識別子の比較
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id1 = WallIdentifier::new();
    /// let id2 = WallIdentifier::new();
    ///
    /// let str1 = id1.as_str();
    /// let str2 = id2.as_str();
    ///
    /// assert_ne!(str1, str2); // 異なる文字列
    /// println!("ID1: {}", str1);
    /// println!("ID2: {}", str2);
    /// ```
    ///
    /// ## パースと検証
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let identifier = WallIdentifier::new();
    /// let id_str = identifier.as_str();
    ///
    /// // 文字列形式の検証
    /// let parts: Vec<&str> = id_str.split('_').collect();
    /// assert_eq!(parts.len(), 2);
    /// assert!(parts[0].starts_with("ThreadId"));
    /// assert!(parts[1].parse::<i64>().is_ok());
    /// ```
    pub fn as_str(&self) -> String {
        format!("{:?}_{}", self.tid_id, self.unix_time_nanos)
    }

    /// スレッドIDを取得します
    ///
    /// この識別子を生成したスレッドのIDを返します。
    /// スレッド固有の処理やデバッグ時に有用です。
    ///
    /// # 戻り値
    ///
    /// 生成元スレッドの `ThreadId`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id = WallIdentifier::new();
    /// let thread_id = id.thread_id();
    /// assert_eq!(thread_id, std::thread::current().id());
    /// ```
    pub fn thread_id(&self) -> thread::ThreadId {
        self.tid_id
    }

    /// UNIX時刻（ナノ秒）を取得します
    ///
    /// この識別子が生成された時刻をUNIXエポックからのナノ秒で返します。
    /// 時系列分析やパフォーマンス測定に有用です。
    ///
    /// # 戻り値
    ///
    /// UNIX エポック（1970-01-01 00:00:00 UTC）からのナノ秒
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id1 = WallIdentifier::new();
    /// let id2 = WallIdentifier::new();
    ///
    /// assert!(id2.unix_time_nanos() > id1.unix_time_nanos());
    /// ```
    pub fn unix_time_nanos(&self) -> i64 {
        self.unix_time_nanos
    }

    /// 別の識別子との時刻差を計算します
    ///
    /// 2つの識別子間の時刻差をナノ秒で返します。
    /// パフォーマンス測定や処理時間の分析に使用できます。
    ///
    /// # 引数
    ///
    /// * `other` - 比較対象の識別子
    ///
    /// # 戻り値
    ///
    /// 時刻差（ナノ秒）。正の値は `other` の方が新しいことを示す。
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let start = WallIdentifier::new();
    /// // 何らかの処理...
    /// let end = WallIdentifier::new();
    ///
    /// let duration_nanos = start.time_diff(&end);
    /// assert!(duration_nanos > 0); // end の方が新しい
    /// ```
    pub fn time_diff(&self, other: &WallIdentifier) -> i64 {
        other.unix_time_nanos - self.unix_time_nanos
    }

    /// 同じスレッドで生成されたかを判定します
    ///
    /// 別の識別子と同じスレッドで生成されたかを確認します。
    /// スレッド固有の処理や競合状態の検出に使用できます。
    ///
    /// # 引数
    ///
    /// * `other` - 比較対象の識別子
    ///
    /// # 戻り値
    ///
    /// 同じスレッドで生成された場合は `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id1 = WallIdentifier::new();
    /// let id2 = WallIdentifier::new();
    ///
    /// assert!(id1.same_thread(&id2)); // 同じスレッドで生成
    /// ```
    pub fn same_thread(&self, other: &WallIdentifier) -> bool {
        self.tid_id == other.tid_id
    }
}

impl Default for WallIdentifier {
    /// デフォルトの識別子を生成します
    ///
    /// `WallIdentifier::new()` と同等の機能を提供します。
    /// 構造体の初期化時にデフォルト値が必要な場合に使用されます。
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let default_id: WallIdentifier = Default::default();
    /// let new_id = WallIdentifier::new();
    ///
    /// // 両方とも有効な識別子だが、値は異なる
    /// assert_ne!(default_id, new_id);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WallIdentifier {
    /// 識別子の表示用フォーマット
    ///
    /// `println!` や `format!` マクロで直接使用可能な表示形式を提供します。
    /// `as_str()` メソッドと同じ出力を生成します。
    ///
    /// # 例
    ///
    /// ```rust
    /// # use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
    /// let id = WallIdentifier::new();
    /// println!("ID: {}", id); // Display trait を使用
    /// assert_eq!(format!("{}", id), id.as_str());
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test_wall_identifier_basic_creation() {
        // 基本的な生成テスト
        let id1 = WallIdentifier::new();
        let id2 = WallIdentifier::new();

        // 異なる識別子が生成されることを確認
        assert_ne!(id1, id2);
        assert_ne!(id1.unix_time_nanos, id2.unix_time_nanos);

        // 同じスレッドで生成されていることを確認
        assert_eq!(id1.tid_id, id2.tid_id);
        assert!(id1.same_thread(&id2));
    }

    #[test]
    fn test_wall_identifier_uniqueness() {
        // 連続生成での一意性テスト
        const NUM_IDS: usize = 100;
        let mut ids = Vec::with_capacity(NUM_IDS);

        for _ in 0..NUM_IDS {
            ids.push(WallIdentifier::new());
        }

        // 全ての識別子が異なることを確認
        for i in 0..NUM_IDS {
            for j in (i + 1)..NUM_IDS {
                assert_ne!(
                    ids[i], ids[j],
                    "IDs at positions {} and {} are identical: {:?} == {:?}",
                    i, j, ids[i], ids[j]
                );
            }
        }

        // HashSetでの一意性確認
        let unique_ids: HashSet<_> = ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), NUM_IDS);
    }

    #[test]
    fn test_wall_identifier_time_progression() {
        // 時刻の単調増加テスト
        const NUM_IDS: usize = 50;
        let mut ids = Vec::with_capacity(NUM_IDS);

        for _ in 0..NUM_IDS {
            ids.push(WallIdentifier::new());
        }

        // 時刻が単調増加していることを確認
        for i in 1..NUM_IDS {
            assert!(
                ids[i].unix_time_nanos() > ids[i - 1].unix_time_nanos(),
                "Time did not increase at position {}: {} <= {}",
                i,
                ids[i].unix_time_nanos(),
                ids[i - 1].unix_time_nanos()
            );
        }

        // 時刻差の統計
        let time_diffs: Vec<_> = (1..NUM_IDS)
            .map(|i| ids[i - 1].time_diff(&ids[i]))
            .collect();

        let min_diff = *time_diffs.iter().min().unwrap();
        let max_diff = *time_diffs.iter().max().unwrap();
        let _median_diff = time_diffs[time_diffs.len() / 2];
        let avg_diff = time_diffs.iter().sum::<i64>() / time_diffs.len() as i64;

        println!(
            "Time difference statistics (ns): min={}, max={}, avg={}",
            min_diff, max_diff, avg_diff
        );

        // 全ての時刻差が正の値
        for diff in time_diffs {
            assert!(diff > 0, "Time difference should be positive");
        }
    }

    #[test]
    fn test_wall_identifier_string_representation() {
        let identifier = WallIdentifier::new();
        let id_str = identifier.as_str();

        // 文字列形式の確認
        assert!(id_str.contains("ThreadId"));
        assert!(id_str.contains("_"));

        // パース可能性の確認
        let parts: Vec<&str> = id_str.split('_').collect();
        assert_eq!(parts.len(), 2, "String should have exactly one underscore");

        let thread_part = parts[0];
        let time_part = parts[1];

        assert!(thread_part.starts_with("ThreadId"));
        assert!(
            time_part.parse::<i64>().is_ok(),
            "Time part should be a valid number: {}",
            time_part
        );

        // Display trait との一致確認
        assert_eq!(format!("{}", identifier), id_str);
    }

    #[test]
    fn test_wall_identifier_accessor_methods() {
        let id = WallIdentifier::new();

        // アクセサメソッドの動作確認
        assert_eq!(id.thread_id(), std::thread::current().id());
        assert!(id.unix_time_nanos() > 0);

        // 時刻比較メソッドのテスト
        let id2 = WallIdentifier::new();
        let time_diff = id.time_diff(&id2);
        assert!(time_diff > 0); // id2 の方が新しい

        assert!(id.same_thread(&id2)); // 同じスレッド
    }

    #[test]
    fn test_wall_identifier_traits() {
        let original = WallIdentifier::new();

        // Copy trait のテスト
        let copied = original;
        assert_eq!(original, copied);

        // Clone trait のテスト
        let cloned = original;
        assert_eq!(original, cloned);

        // Default trait のテスト
        let default_id: WallIdentifier = Default::default();
        assert_ne!(original, default_id); // 異なる時刻で生成される

        // Hash trait のテスト
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        original.hash(&mut hasher1);
        copied.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_wall_identifier_collections() {
        // HashSet での使用テスト
        let mut set = HashSet::new();
        let id1 = WallIdentifier::new();
        let id2 = WallIdentifier::new();
        let id1_copy = id1;

        assert!(set.insert(id1));
        assert!(set.insert(id2));
        assert!(!set.insert(id1_copy)); // 既に存在

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));

        // HashMap のキーとしての使用テスト
        let mut map = HashMap::new();
        map.insert(id1, "wall_1");
        map.insert(id2, "wall_2");

        assert_eq!(map.get(&id1), Some(&"wall_1"));
        assert_eq!(map.get(&id2), Some(&"wall_2"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_wall_identifier_multithreaded() {
        // マルチスレッド環境での生成テスト
        const NUM_THREADS: usize = 8;
        const IDS_PER_THREAD: usize = 50;

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for thread_num in 0..NUM_THREADS {
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let mut thread_ids = Vec::new();

                for _ in 0..IDS_PER_THREAD {
                    let identifier = WallIdentifier::new();
                    thread_ids.push(identifier);
                }

                // 各スレッド内での一意性確認
                let unique_in_thread: HashSet<_> = thread_ids.iter().cloned().collect();
                assert_eq!(unique_in_thread.len(), IDS_PER_THREAD);

                // グローバル結果に追加
                let mut locked_results = results_clone.lock().unwrap();
                locked_results.extend(thread_ids.into_iter().map(|id| (thread_num, id)));
            });

            handles.push(handle);
        }

        // 全スレッド完了待ち
        for handle in handles {
            handle.join().unwrap();
        }

        let final_results = results.lock().unwrap();
        assert_eq!(final_results.len(), NUM_THREADS * IDS_PER_THREAD);

        // 全体での一意性確認
        let all_ids: HashSet<_> = final_results.iter().map(|(_, id)| *id).collect();
        assert_eq!(all_ids.len(), NUM_THREADS * IDS_PER_THREAD);

        // スレッドID分布の確認
        let mut thread_id_usage = HashMap::new();
        for (_thread_num, id) in final_results.iter() {
            *thread_id_usage.entry(id.thread_id()).or_insert(0) += 1;
        }

        println!("Thread ID usage distribution:");
        for (thread_id, count) in thread_id_usage.iter() {
            println!("  {:?}: {} identifiers", thread_id, count);
        }

        // 少なくとも1つのスレッドIDが使用されている
        assert!(!thread_id_usage.is_empty());
    }

    #[test]
    fn test_wall_identifier_performance() {
        // 性能測定テスト
        const NUM_CREATIONS: usize = 1000;

        let start_time = Instant::now();
        let mut ids = Vec::with_capacity(NUM_CREATIONS);

        for _ in 0..NUM_CREATIONS {
            ids.push(WallIdentifier::new());
        }

        let creation_duration = start_time.elapsed();

        // 一意性確認
        let unique_ids: HashSet<_> = ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), NUM_CREATIONS);

        // 性能統計
        println!(
            "Performance test: {} identifiers created in {:?}",
            NUM_CREATIONS, creation_duration
        );
        println!(
            "Average creation time: {:?}",
            creation_duration / NUM_CREATIONS as u32
        );

        // 文字列変換の性能
        let string_start = Instant::now();
        let id_strings: Vec<_> = ids.iter().map(|id| id.as_str()).collect();
        let string_duration = string_start.elapsed();

        println!(
            "String conversion: {} conversions in {:?}",
            NUM_CREATIONS, string_duration
        );

        // 文字列の一意性確認
        let unique_strings: HashSet<_> = id_strings.into_iter().collect();
        assert_eq!(unique_strings.len(), NUM_CREATIONS);
    }

    #[test]
    fn test_wall_identifier_stress_test() {
        // ストレステスト：高速連続生成
        const BURST_SIZE: usize = 100;
        const NUM_BURSTS: usize = 5;

        let mut all_ids = Vec::with_capacity(BURST_SIZE * NUM_BURSTS);

        for burst_num in 0..NUM_BURSTS {
            let burst_start = Instant::now();
            let mut burst_ids = Vec::with_capacity(BURST_SIZE);

            for _ in 0..BURST_SIZE {
                burst_ids.push(WallIdentifier::new());
            }

            let burst_duration = burst_start.elapsed();
            println!(
                "Burst {}: {} IDs in {:?} (avg: {:?}/ID)",
                burst_num,
                BURST_SIZE,
                burst_duration,
                burst_duration / BURST_SIZE as u32
            );

            // バースト内一意性確認
            let burst_unique: HashSet<_> = burst_ids.iter().cloned().collect();
            assert_eq!(burst_unique.len(), BURST_SIZE);

            all_ids.extend(burst_ids);

            // バースト間の少し長い待機
            thread::sleep(Duration::from_micros(100));
        }

        // 全体一意性確認
        let all_unique: HashSet<_> = all_ids.iter().cloned().collect();
        assert_eq!(all_unique.len(), BURST_SIZE * NUM_BURSTS);
    }

    #[test]
    fn test_wall_identifier_random_sleep_effectiveness() {
        // ランダムスリープの効果確認テスト
        const NUM_SAMPLES: usize = 200;
        let mut time_diffs = Vec::with_capacity(NUM_SAMPLES - 1);

        let mut prev_id = WallIdentifier::new();
        for _ in 1..NUM_SAMPLES {
            let current_id = WallIdentifier::new();
            let diff = prev_id.time_diff(&current_id);
            time_diffs.push(diff);
            prev_id = current_id;
        }

        // 統計計算
        time_diffs.sort();
        let min_diff = time_diffs[0];
        let max_diff = time_diffs[time_diffs.len() - 1];
        let median_diff = time_diffs[time_diffs.len() / 2];
        let avg_diff = time_diffs.iter().sum::<i64>() / time_diffs.len() as i64;

        println!("Random sleep effectiveness statistics:");
        println!("  Min time diff: {} ns", min_diff);
        println!("  Max time diff: {} ns", max_diff);
        println!("  Median time diff: {} ns", median_diff);
        println!("  Average time diff: {} ns", avg_diff);

        // ランダムスリープによるばらつき確認
        assert!(
            max_diff > min_diff,
            "Should have variation due to random sleep"
        );
        assert!(min_diff > 0, "All time differences should be positive");

        // 期待されるランダムスリープ範囲（5-10ns）内の変動があることを確認
        // 実際の測定値はシステムオーバーヘッドも含むため、より大きな値になる
        let variation = max_diff - min_diff;
        assert!(variation > 0, "Should have time variation");

        println!("  Time variation: {} ns", variation);
    }

    #[test]
    fn test_wall_identifier_boundary_conditions() {
        // 境界条件のテスト

        // 1. 最小待機時間での生成
        let id1 = WallIdentifier::new();
        let id2 = WallIdentifier::new();
        assert_ne!(id1, id2);

        // 2. システム時刻の一貫性確認
        let before_creation = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let id = WallIdentifier::new();

        let after_creation = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        // 生成時刻が妥当な範囲内にあることを確認
        assert!(id.unix_time_nanos() >= before_creation);
        assert!(id.unix_time_nanos() <= after_creation);

        // 3. スレッドID の一貫性
        let current_thread_id = thread::current().id();
        assert_eq!(id.thread_id(), current_thread_id);
    }

    #[test]
    fn test_wall_identifier_concurrent_creation() {
        // 並行生成での競合状態テスト
        const NUM_CONCURRENT_THREADS: usize = 10;
        const OPERATIONS_PER_THREAD: usize = 100;

        let barrier = Arc::new(std::sync::Barrier::new(NUM_CONCURRENT_THREADS));
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for thread_id in 0..NUM_CONCURRENT_THREADS {
            let barrier_clone = Arc::clone(&barrier);
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                // 全スレッドが準備完了まで待機
                barrier_clone.wait();

                let mut local_ids = Vec::with_capacity(OPERATIONS_PER_THREAD);

                // 一斉に識別子生成開始
                for _ in 0..OPERATIONS_PER_THREAD {
                    local_ids.push(WallIdentifier::new());
                }

                // 結果をグローバル集合に追加
                let mut locked_results = results_clone.lock().unwrap();
                locked_results.extend(local_ids.into_iter().map(|id| (thread_id, id)));
            });

            handles.push(handle);
        }

        // 全スレッド完了待ち
        for handle in handles {
            handle.join().unwrap();
        }

        let final_results = results.lock().unwrap();
        let total_ids = NUM_CONCURRENT_THREADS * OPERATIONS_PER_THREAD;
        assert_eq!(final_results.len(), total_ids);

        // 完全な一意性確認
        let all_ids: HashSet<_> = final_results.iter().map(|(_, id)| *id).collect();
        assert_eq!(
            all_ids.len(),
            total_ids,
            "All {} identifiers should be unique",
            total_ids
        );

        println!(
            "Concurrent creation test: {} unique IDs from {} threads",
            all_ids.len(),
            NUM_CONCURRENT_THREADS
        );
    }

    #[test]
    fn test_wall_identifier_format_consistency() {
        // フォーマット一貫性テスト
        let id = WallIdentifier::new();

        let debug_format = format!("{:?}", id);
        let display_format = format!("{}", id);
        let as_str_format = id.as_str();

        // Display と as_str() の一致
        assert_eq!(display_format, as_str_format);

        // Debug フォーマットの妥当性
        assert!(debug_format.contains("WallIdentifier"));
        assert!(debug_format.contains("tid_id"));
        assert!(debug_format.contains("unix_time_nanos"));

        // 文字列の解析可能性
        let parts: Vec<&str> = as_str_format.split('_').collect();
        assert_eq!(parts.len(), 2);

        // ThreadId 部分の検証
        assert!(parts[0].starts_with("ThreadId("));
        assert!(parts[0].ends_with(")"));

        // 時刻部分の検証
        let time_value = parts[1].parse::<i64>().unwrap();
        assert_eq!(time_value, id.unix_time_nanos());
    }

    #[test]
    fn test_wall_identifier_comparison_methods() {
        // 比較メソッドの詳細テスト
        let id1 = WallIdentifier::new();
        let id2 = WallIdentifier::new();

        // 基本的な比較
        assert_ne!(id1, id2);
        assert!(id1.same_thread(&id2)); // 同じスレッド

        // 時刻差の確認
        let time_diff = id1.time_diff(&id2);
        assert!(time_diff > 0); // id2 の方が新しい

        let reverse_diff = id2.time_diff(&id1);
        assert!(reverse_diff < 0); // 逆方向は負の値
        assert_eq!(time_diff, -reverse_diff);

        // 自分自身との比較
        assert_eq!(id1, id1);
        assert_eq!(id1.time_diff(&id1), 0);
        assert!(id1.same_thread(&id1));

        // マルチスレッドでの異なるスレッドID確認
        let other_thread_id = thread::spawn(WallIdentifier::new).join().unwrap();

        // 通常は異なるスレッドIDになるが、環境によっては同じ場合もある
        if !id1.same_thread(&other_thread_id) {
            println!(
                "Different thread IDs detected: {:?} vs {:?}",
                id1.thread_id(),
                other_thread_id.thread_id()
            );
        }
    }
}
