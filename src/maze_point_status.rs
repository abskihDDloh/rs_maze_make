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
/// # 使用例
///
/// ```rust
/// let identifier = WallIdentifier::new();
/// println!("識別子: {}", identifier.as_str()); // "ThreadId(...)_1633036800123456789"
/// ```
///
/// # 注意事項
///
/// - 生成時に10ナノ秒のスリープを行うため、高頻度での生成は性能に影響する可能性があります
/// - 時刻が逆行した場合はパニックします（通常は発生しません）
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub(crate) struct WallIdentifier {
    tid_id: thread::ThreadId,
    unix_time_nanos: i64,
}

impl WallIdentifier {
    /// 新しい壁識別子を生成します
    ///
    /// 生成される識別子は、現在のスレッドIDと現在時刻のナノ秒を組み合わせた
    /// 形式になります。重複を回避するため、生成時に短時間のスリープを行います。
    ///
    /// # 戻り値
    ///
    /// 新しい `WallIdentifier` インスタンス
    ///
    /// # パニック
    ///
    /// システム時刻が UNIX エポック以前に設定されている場合にパニックします
    ///
    /// # 例
    ///
    /// ```rust
    /// let id1 = WallIdentifier::new();
    /// let id2 = WallIdentifier::new();
    /// assert_ne!(id1, id2); // 異なる識別子が生成される
    /// ```
    pub(crate) fn new() -> Self {
        // 10nsのスリープを入れて、UNIX時間の重複を回避する
        use std::time::Duration;
        thread::sleep(Duration::new(0, 10));
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
    /// 形式: "ThreadId(...)_<ナノ秒時刻>"
    ///
    /// # 戻り値
    ///
    /// 識別子の文字列表現
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let id_str = identifier.as_str();
    /// println!("ID: {}", id_str); // "ThreadId(1)_1633036800123456789"
    /// ```
    pub fn as_str(&self) -> String {
        format!("{:?}_{}", self.tid_id, self.unix_time_nanos)
    }
}

/// 柱の拡張状態を表す列挙型
///
/// 迷路生成アルゴリズムにおいて、各柱の処理状態を管理するために使用されます。
/// この状態遷移により、迷路生成の進行状況を追跡できます。
///
/// # 状態遷移
///
/// ```text
/// NotChecked → Extending
/// ```
///
/// - `NotChecked`: 初期状態。まだ処理されていない柱
/// - `Extending`: 拡張処理中。他の柱への接続を試行している状態
///
/// # 識別子の関係
///
/// - `NotChecked` 状態の柱は識別子を持ちません
/// - `Extending` 状態の柱は処理を開始したスレッドの識別子を持ちます
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PillarExtendStatus {
    /// 拡張処理未開始
    ///
    /// この状態の柱は：
    /// - まだどのスレッドからも処理されていない
    /// - 識別子は None
    /// - 開始点の候補として選択可能
    NotChecked,

    /// 拡張処理中
    ///
    /// この状態の柱は：
    /// - 特定のスレッドが処理を開始している
    /// - そのスレッドの識別子を持つ
    /// - 他の隣接柱への拡張を試行中
    Extending,
}

/// 壁の種類を表す列挙型
///
/// 迷路の各座標点における壁の性質を分類します。
/// この分類により、迷路生成アルゴリズムは適切な処理を選択できます。
///
/// # 壁の種類
///
/// - `Outside`: 迷路の境界を形成する外壁
/// - `Pillar`: 迷路内部の柱（壁生成の起点）
/// - `Wall`: 柱から拡張されて生成された壁
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum WallType {
    /// 外壁（迷路の境界）
    ///
    /// 特徴：
    /// - 迷路の最外周に配置される
    /// - 常に固定的な壁として機能
    /// - 初期化時に配置される
    /// - 識別子を持つ（境界の識別子）
    Outside,

    /// 柱（壁生成の起点）
    ///
    /// 特徴：
    /// - 迷路内部の偶数座標に配置される
    /// - 拡張状態を持つ（NotChecked または Extending）
    /// - NotChecked 時は識別子なし、Extending 時は識別子あり
    /// - 他の柱への接続を試行する起点となる
    Pillar(PillarExtendStatus),

    /// 生成された壁
    ///
    /// 特徴：
    /// - 柱から柱への拡張時に中間点に生成される
    /// - 生成したスレッドの識別子を持つ
    /// - 迷路の内部構造を形成する
    Wall,
}

impl WallType {
    /// NotChecked状態の柱からExtending状態の柱に変換します
    ///
    /// この関数は、未チェック状態の柱を拡張処理中の状態に変更します。
    /// 入力が適切なNotChecked柱でない場合はエラーを返します。
    ///
    /// # 引数
    ///
    /// * `src_wall` - 変換元の壁タイプ（NotChecked柱である必要があります）
    ///
    /// # 戻り値
    ///
    /// 成功した場合はExtending状態の柱、失敗した場合はエラー
    ///
    /// # エラー
    ///
    /// * 入力がNotChecked状態の柱でない場合
    ///
    /// # 例
    ///
    /// ```rust
    /// let notchecked = WallType::Pillar(PillarExtendStatus::NotChecked);
    /// let extending = WallType::new_extending_pillar_from_notchecked_pillar(notchecked)?;
    /// assert!(matches!(extending, WallType::Pillar(PillarExtendStatus::Extending)));
    /// ```
    pub(crate) fn new_extending_pillar_from_notchecked_pillar(
        src_wall: WallType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_wall {
            WallType::Pillar(PillarExtendStatus::NotChecked) => {
                Ok(WallType::Pillar(PillarExtendStatus::Extending))
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Cannot convert to extending pillar: expected NotChecked pillar, found {:?}",
                    src_wall
                ),
            ))),
        }
    }
}

/// 迷路の各座標点の状態を表す列挙型
///
/// 迷路の各座標は通路か壁のいずれかの状態を持ちます。
/// この分類により、迷路生成アルゴリズムと最終的な迷路表示が実現されます。
///
/// # 座標点の状態
///
/// - `Path`: 通路として利用可能な空間
/// - `Wall`: 何らかの壁（外壁、柱、生成された壁）
///
/// # 識別子システム
///
/// 壁は識別子を持つことで、どのスレッドの処理で生成されたかを追跡できます。
/// これにより、マルチスレッド環境での迷路生成が安全に実行されます。
///
/// # 判定メソッド
///
/// 各状態を効率的に判定するためのメソッドが提供されています：
/// - `is_outside_wall()`: 外壁かどうかの判定
/// - `is_not_checked_wall()`: 未チェック柱かどうかの判定
/// - `is_extending_wall()`: 拡張中柱かどうかの判定
/// - `is_my_wall()`: 特定の識別子で生成された壁かどうかの判定
///
/// # 状態変換メソッド
///
/// 柱の状態を安全に変換するためのメソッドが提供されています：
/// - `new_extending_pillar_from_notchecked_pillar()`: NotChecked柱をExtending柱に変換
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MazePointStatus {
    /// 通路
    ///
    /// 特徴：
    /// - プレイヤーが移動可能な空間
    /// - 初期化時にほとんどの座標がこの状態
    /// - 壁の生成により一部が壁に変更される
    Path,

    /// 壁
    ///
    /// 特徴：
    /// - 移動不可能な障害物
    /// - 壁の種類（WallType）と識別子を持つ
    /// - 識別子により生成元のスレッドを特定可能
    Wall(WallType, Option<WallIdentifier>),
}

impl MazePointStatus {
    /// 外壁を生成します
    ///
    /// 迷路の境界に配置される外壁のステータスを作成します。
    /// 外壁は必ず識別子を持ちます。
    ///
    /// # 引数
    ///
    /// * `identifier` - 外壁の識別子
    ///
    /// # 戻り値
    ///
    /// 外壁を表す `MazePointStatus`
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let outside_wall = MazePointStatus::new_outside_wall(identifier);
    /// assert!(outside_wall.is_outside_wall());
    /// ```
    pub fn new_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(WallType::Outside, Some(identifier))
    }

    /// 未チェック状態の柱を生成します
    ///
    /// 迷路内部に配置される初期状態の柱のステータスを作成します。
    /// 未チェック状態の柱は識別子を持ちません。
    ///
    /// # 戻り値
    ///
    /// 未チェック状態の柱を表す `MazePointStatus`
    ///
    /// # 例
    ///
    /// ```rust
    /// let pillar = MazePointStatus::new_notchecked_pillar();
    /// assert!(pillar.is_not_checked_wall());
    /// ```
    pub fn new_notchecked_pillar() -> Self {
        MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None)
    }

    /// 未チェック状態の柱から拡張中状態の柱に変換します
    ///
    /// この関数は、未チェック状態の柱を拡張処理中の状態に変更します。
    /// 内部的に`WallType::new_extending_pillar_from_notchecked_pillar()`を使用して
    /// 型安全な変換を行います。
    ///
    /// # 引数
    ///
    /// * `src_pillar` - 変換元の柱の状態（NotChecked柱である必要があります）
    /// * `identifier` - 拡張中状態の柱に設定する識別子
    ///
    /// # 戻り値
    ///
    /// 成功した場合は拡張中状態の柱、失敗した場合はエラー
    ///
    /// # エラー
    ///
    /// * 入力がNotChecked状態の柱でない場合
    /// * 入力の識別子がNoneでない場合
    /// * WallTypeの変換に失敗した場合
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let notchecked = MazePointStatus::new_notchecked_pillar();
    /// let extending = MazePointStatus::new_extending_pillar_from_notchecked_pillar(notchecked, &identifier)?;
    /// assert!(extending.is_extending_wall());
    /// assert!(extending.is_my_wall(&identifier));
    /// ```
    pub fn new_extending_pillar_from_notchecked_pillar(
        src_pillar: MazePointStatus,
        identifier: &WallIdentifier,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_pillar {
            MazePointStatus::Wall(wall_type, None) => {
                // WallType::new_extending_pillar_from_notchecked_pillar() を使用して変換
                let extending_wall_type =
                    WallType::new_extending_pillar_from_notchecked_pillar(wall_type)?;

                Ok(MazePointStatus::Wall(
                    extending_wall_type,
                    Some(*identifier),
                ))
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Cannot convert to extending pillar: expected NotChecked pillar with no identifier, found {:?}",
                    src_pillar
                ),
            ))),
        }
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, MazePointStatus::Wall(..))
    }

    pub fn is_path(&self) -> bool {
        matches!(self, MazePointStatus::Path)
    }

    pub fn get_wall_identifier(&self) -> Option<&WallIdentifier> {
        match self {
            MazePointStatus::Wall(_, Some(id)) => Some(id),
            _ => None,
        }
    }

    /// この座標点が外壁かどうかを判定します
    ///
    /// 外壁は迷路の境界に配置される固定的な壁です。
    /// この判定は迷路生成アルゴリズムで境界処理を行う際に使用されます。
    ///
    /// # 戻り値
    ///
    /// 外壁の場合は `true`、そうでなければ `false`
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let outside_wall = MazePointStatus::new_outside_wall(identifier);
    /// assert!(outside_wall.is_outside_wall());
    ///
    /// let path = MazePointStatus::Path;
    /// assert!(!path.is_outside_wall());
    /// ```
    pub fn is_outside_wall(&self) -> bool {
        matches!(self, MazePointStatus::Wall(WallType::Outside, _))
    }

    /// この座標点が未チェック状態の柱かどうかを判定します
    ///
    /// 未チェック柱は迷路生成の開始点候補として使用される柱です。
    /// この判定は利用可能な開始点を選択する際に使用されます。
    ///
    /// # 戻り値
    ///
    /// 未チェック柱の場合は `true`、そうでなければ `false`
    ///
    /// # 例
    ///
    /// ```rust
    /// let pillar = MazePointStatus::new_notchecked_pillar();
    /// assert!(pillar.is_not_checked_wall());
    ///
    /// let identifier = WallIdentifier::new();
    /// let extending_pillar = MazePointStatus::Wall(
    ///     WallType::Pillar(PillarExtendStatus::Extending),
    ///     Some(identifier)
    /// );
    /// assert!(!extending_pillar.is_not_checked_wall());
    /// ```
    pub fn is_not_checked_wall(&self) -> bool {
        matches!(
            self,
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None)
        )
    }

    /// この座標点が拡張中状態の柱かどうかを判定します
    ///
    /// 拡張中柱は現在処理中の柱で、隣接する柱への接続を試行している状態です。
    /// この判定は迷路生成の進行状況を追跡する際に使用されます。
    ///
    /// # 戻り値
    ///
    /// 拡張中柱の場合は `true`、そうでなければ `false`
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let extending_pillar = MazePointStatus::Wall(
    ///     WallType::Pillar(PillarExtendStatus::Extending),
    ///     Some(identifier)
    /// );
    /// assert!(extending_pillar.is_extending_wall());
    ///
    /// let notchecked_pillar = MazePointStatus::new_notchecked_pillar();
    /// assert!(!notchecked_pillar.is_extending_wall());
    /// ```
    pub fn is_extending_wall(&self) -> bool {
        matches!(
            self,
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(_))
        )
    }

    /// 指定された識別子で生成された壁かどうかを判定します
    ///
    /// この関数は、特定のスレッドや処理で生成された壁を識別するために
    /// 使用されます。マルチスレッド環境での所有権確認に有用です。
    ///
    /// # 引数
    ///
    /// * `identifier` - 確認したい識別子
    ///
    /// # 戻り値
    ///
    /// 指定された識別子で生成された壁の場合は `true`、そうでなければ `false`
    ///
    /// # 例
    ///
    /// ```rust
    /// let identifier = WallIdentifier::new();
    /// let wall = MazePointStatus::new_outside_wall(identifier);
    /// assert!(wall.is_my_wall(&identifier));
    ///
    /// let other_identifier = WallIdentifier::new();
    /// assert!(!wall.is_my_wall(&other_identifier));
    /// ```
    pub fn is_my_wall(&self, identifier: &WallIdentifier) -> bool {
        matches!(self, MazePointStatus::Wall(_, Some(id)) if id == identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 基本的な Path 状態のテスト
    #[test]
    fn test_maze_point_status_path() {
        let status = MazePointStatus::Path;
        assert!(matches!(status, MazePointStatus::Path));
        assert!(!matches!(status, MazePointStatus::Wall(_, _)));
    }

    /// WallIdentifier の一意性と形式のテスト
    #[test]
    fn test_wall_identifier_new_and_as_str() {
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 異なる識別子が生成されることを確認
        assert_ne!(identifier1.as_str(), identifier2.as_str());

        // 識別子が期待する形式であることを確認
        let id_str = identifier1.as_str();
        assert!(id_str.contains("ThreadId"));
        assert!(id_str.contains("_"));
    }

    /// 外壁の生成と構造のテスト
    #[test]
    fn test_maze_point_status_new_outside_wall() {
        let identifier = WallIdentifier::new();
        let status = MazePointStatus::new_outside_wall(identifier);

        assert!(matches!(status, MazePointStatus::Wall(_, _)));
        assert!(!matches!(status, MazePointStatus::Path));

        match status {
            MazePointStatus::Wall(WallType::Outside, Some(id)) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Wall with Outside type"),
        }
    }

    /// 未チェック柱の生成と構造のテスト
    #[test]
    fn test_maze_point_status_new_notchecked_pillar() {
        let status = MazePointStatus::new_notchecked_pillar();

        match status {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None) => {
                // 正常: NotChecked柱は識別子がNone
            }
            _ => panic!("Expected Wall with NotChecked Pillar and None identifier"),
        }
    }

    /// WallType::new_extending_pillar_from_notchecked_pillar メソッドのテスト
    #[test]
    fn test_wall_type_new_extending_pillar_from_notchecked_pillar() {
        // 正常ケース: NotChecked柱からExtending柱への変換
        let notchecked_wall = WallType::Pillar(PillarExtendStatus::NotChecked);
        let result = WallType::new_extending_pillar_from_notchecked_pillar(notchecked_wall);

        assert!(result.is_ok());
        let extending_wall = result.unwrap();

        match extending_wall {
            WallType::Pillar(PillarExtendStatus::Extending) => {
                // 正常: Extending状態
            }
            _ => panic!("Expected Extending pillar"),
        }
    }

    /// WallType::new_extending_pillar_from_notchecked_pillar エラーケースのテスト
    #[test]
    fn test_wall_type_new_extending_pillar_error_cases() {
        // エラーケース1: Outside壁
        let outside_wall = WallType::Outside;
        let result = WallType::new_extending_pillar_from_notchecked_pillar(outside_wall);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース2: Extending状態の柱
        let extending_wall = WallType::Pillar(PillarExtendStatus::Extending);
        let result = WallType::new_extending_pillar_from_notchecked_pillar(extending_wall);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース3: 通常の壁
        let normal_wall = WallType::Wall;
        let result = WallType::new_extending_pillar_from_notchecked_pillar(normal_wall);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );
    }

    /// MazePointStatus::new_extending_pillar_from_notchecked_pillar メソッドのテスト
    /// WallType::new_extending_pillar_from_notchecked_pillar() を内部的に使用することを確認
    #[test]
    fn test_maze_point_status_new_extending_pillar_from_notchecked_pillar() {
        let identifier = WallIdentifier::new();

        // 正常ケース: NotChecked柱からExtending柱への変換
        let notchecked_pillar = MazePointStatus::new_notchecked_pillar();
        let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            notchecked_pillar,
            &identifier,
        );

        assert!(result.is_ok());
        let extending_pillar = result.unwrap();

        match extending_pillar {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(id)) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Extending pillar with identifier"),
        }

        // 新しいメソッドでの確認
        assert!(extending_pillar.is_extending_wall());
        assert!(extending_pillar.is_my_wall(&identifier));
        assert!(!extending_pillar.is_not_checked_wall());
        assert!(!extending_pillar.is_outside_wall());
    }

    /// MazePointStatus::new_extending_pillar_from_notchecked_pillar エラーケースのテスト
    /// WallType変換のエラーも含めて確認
    #[test]
    fn test_maze_point_status_new_extending_pillar_error_cases() {
        let identifier = WallIdentifier::new();

        // エラーケース1: Path状態（MazePointStatusレベルでのエラー）
        let path = MazePointStatus::Path;
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(path, &identifier);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース2: Outside壁（WallTypeレベルでのエラー）
        let outside_wall = MazePointStatus::Wall(WallType::Outside, None);
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(outside_wall, &identifier);
        assert!(result.is_err());
        // WallType::new_extending_pillar_from_notchecked_pillar() からのエラーメッセージを確認
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース3: Extending状態の柱（WallTypeレベルでのエラー）
        let extending_wall =
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), None);
        let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            extending_wall,
            &identifier,
        );
        assert!(result.is_err());
        // WallType::new_extending_pillar_from_notchecked_pillar() からのエラーメッセージを確認
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース4: NotChecked柱だが識別子あり（MazePointStatusレベルでのエラー）
        let invalid_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::NotChecked),
            Some(identifier),
        );
        let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            invalid_pillar,
            &identifier,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // エラーケース5: 通常の壁（WallTypeレベルでのエラー）
        let normal_wall = MazePointStatus::Wall(WallType::Wall, None);
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(normal_wall, &identifier);
        assert!(result.is_err());
        // WallType::new_extending_pillar_from_notchecked_pillar() からのエラーメッセージを確認
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );
    }

    /// WallType変換メソッドとMazePointStatus変換メソッドの連携テスト
    #[test]
    fn test_wall_type_and_maze_point_status_conversion_integration() {
        let identifier = WallIdentifier::new();

        // WallTypeレベルでの変換
        let notchecked_wall_type = WallType::Pillar(PillarExtendStatus::NotChecked);
        let extending_wall_type_result =
            WallType::new_extending_pillar_from_notchecked_pillar(notchecked_wall_type);
        assert!(extending_wall_type_result.is_ok());

        // MazePointStatusレベルでの変換（内部的にWallType変換を使用）
        let notchecked_pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar_result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            notchecked_pillar,
            &identifier,
        );
        assert!(extending_pillar_result.is_ok());

        // 結果の一貫性確認
        let extending_wall_type = extending_wall_type_result.unwrap();
        let extending_pillar = extending_pillar_result.unwrap();

        match (extending_wall_type, extending_pillar) {
            (
                WallType::Pillar(PillarExtendStatus::Extending),
                MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(_)),
            ) => {
                // 正常: 両方ともExtending状態
            }
            _ => panic!("Expected consistent Extending state"),
        }
    }

    /// エラー伝播のテスト（WallTypeからMazePointStatusへ）
    #[test]
    fn test_error_propagation_from_wall_type_to_maze_point_status() {
        let identifier = WallIdentifier::new();

        // WallTypeレベルで失敗するケース
        let invalid_wall_types = vec![
            WallType::Outside,
            WallType::Wall,
            WallType::Pillar(PillarExtendStatus::Extending),
        ];

        for invalid_wall_type in invalid_wall_types {
            // WallTypeレベルでの直接変換（失敗するはず）
            let wall_type_result =
                WallType::new_extending_pillar_from_notchecked_pillar(invalid_wall_type.clone());
            assert!(wall_type_result.is_err());

            // MazePointStatusレベルでの変換（WallTypeエラーが伝播するはず）
            let invalid_maze_point = MazePointStatus::Wall(invalid_wall_type, None);
            let maze_point_result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
                invalid_maze_point,
                &identifier,
            );
            assert!(maze_point_result.is_err());

            // エラーメッセージがWallTypeから伝播していることを確認
            let error_msg = maze_point_result.unwrap_err().to_string();
            assert!(error_msg.contains("expected NotChecked pillar"));
        }
    }

    /// is_outside_wall メソッドのテスト
    #[test]
    fn test_is_outside_wall_method() {
        let identifier = WallIdentifier::new();

        // 外壁のテスト
        let outside_wall = MazePointStatus::new_outside_wall(identifier);
        assert!(outside_wall.is_outside_wall());

        // 通路は外壁ではない
        let path = MazePointStatus::Path;
        assert!(!path.is_outside_wall());

        // 未チェック柱は外壁ではない
        let pillar = MazePointStatus::new_notchecked_pillar();
        assert!(!pillar.is_outside_wall());

        // Extending柱は外壁ではない
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );
        assert!(!extending_pillar.is_outside_wall());

        // 生成された壁は外壁ではない
        let wall = MazePointStatus::Wall(WallType::Wall, Some(identifier));
        assert!(!wall.is_outside_wall());
    }

    /// is_not_checked_wall メソッドのテスト
    #[test]
    fn test_is_not_checked_wall_method() {
        let identifier = WallIdentifier::new();

        // 未チェック柱のテスト
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        assert!(not_checked_pillar.is_not_checked_wall());

        // 通路は未チェック柱ではない
        let path = MazePointStatus::Path;
        assert!(!path.is_not_checked_wall());

        // 外壁は未チェック柱ではない
        let outside_wall = MazePointStatus::new_outside_wall(identifier);
        assert!(!outside_wall.is_not_checked_wall());

        // Extending柱は未チェック柱ではない
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );
        assert!(!extending_pillar.is_not_checked_wall());

        // 生成された壁は未チェック柱ではない
        let wall = MazePointStatus::Wall(WallType::Wall, Some(identifier));
        assert!(!wall.is_not_checked_wall());
    }

    /// is_extending_wall メソッドのテスト
    #[test]
    fn test_is_extending_wall_method() {
        let identifier = WallIdentifier::new();

        // Extending柱のテスト
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );
        assert!(extending_pillar.is_extending_wall());

        // 通路はExtending柱ではない
        let path = MazePointStatus::Path;
        assert!(!path.is_extending_wall());

        // 外壁はExtending柱ではない
        let outside_wall = MazePointStatus::new_outside_wall(identifier);
        assert!(!outside_wall.is_extending_wall());

        // 未チェック柱はExtending柱ではない
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        assert!(!not_checked_pillar.is_extending_wall());

        // 生成された壁はExtending柱ではない
        let wall = MazePointStatus::Wall(WallType::Wall, Some(identifier));
        assert!(!wall.is_extending_wall());
    }

    /// 判定メソッドの相互排他性テスト
    #[test]
    fn test_wall_type_exclusivity() {
        let identifier = WallIdentifier::new();

        // 外壁
        let outside_wall = MazePointStatus::new_outside_wall(identifier);
        assert!(outside_wall.is_outside_wall());
        assert!(!outside_wall.is_not_checked_wall());
        assert!(!outside_wall.is_extending_wall());

        // 未チェック柱
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        assert!(!not_checked_pillar.is_outside_wall());
        assert!(not_checked_pillar.is_not_checked_wall());
        assert!(!not_checked_pillar.is_extending_wall());

        // Extending柱
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );
        assert!(!extending_pillar.is_outside_wall());
        assert!(!extending_pillar.is_not_checked_wall());
        assert!(extending_pillar.is_extending_wall());

        // 生成された壁
        let wall = MazePointStatus::Wall(WallType::Wall, Some(identifier));
        assert!(!wall.is_outside_wall());
        assert!(!wall.is_not_checked_wall());
        assert!(!wall.is_extending_wall());

        // 通路
        let path = MazePointStatus::Path;
        assert!(!path.is_outside_wall());
        assert!(!path.is_not_checked_wall());
        assert!(!path.is_extending_wall());
    }

    /// PillarExtendStatus の各状態の直接マッチングテスト
    #[test]
    fn test_pillar_extend_status_direct_matching() {
        let not_checked = PillarExtendStatus::NotChecked;
        let extending = PillarExtendStatus::Extending;

        // NotChecked 状態のテスト
        assert!(matches!(not_checked, PillarExtendStatus::NotChecked));
        assert!(!matches!(not_checked, PillarExtendStatus::Extending));

        // Extending 状態のテスト
        assert!(!matches!(extending, PillarExtendStatus::NotChecked));
        assert!(matches!(extending, PillarExtendStatus::Extending));
    }

    /// WallType の各種類の直接マッチングテスト
    #[test]
    fn test_wall_type_direct_matching() {
        let outside = WallType::Outside;
        let pillar = WallType::Pillar(PillarExtendStatus::NotChecked);
        let wall = WallType::Wall;

        // Outside型のテスト
        assert!(matches!(outside, WallType::Outside));
        assert!(!matches!(outside, WallType::Pillar(_)));
        assert!(!matches!(outside, WallType::Wall));

        // Pillar型のテスト
        assert!(matches!(pillar, WallType::Pillar(_)));
        assert!(!matches!(pillar, WallType::Outside));
        assert!(!matches!(pillar, WallType::Wall));

        // Pillarの内部状態の確認
        if let WallType::Pillar(status) = pillar {
            assert!(matches!(status, PillarExtendStatus::NotChecked));
        }

        // Wall型のテスト
        assert!(matches!(wall, WallType::Wall));
        assert!(!matches!(wall, WallType::Pillar(_)));
        assert!(!matches!(wall, WallType::Outside));
    }

    /// WallIdentifier の一意性を大量生成でテスト
    #[test]
    fn test_wall_identifier_uniqueness() {
        let mut identifiers = Vec::new();

        // 複数の識別子を生成
        for _ in 0..10 {
            identifiers.push(WallIdentifier::new());
        }

        // 全て異なることを確認
        for i in 0..identifiers.len() {
            for j in i + 1..identifiers.len() {
                assert_ne!(identifiers[i].as_str(), identifiers[j].as_str());
            }
        }
    }

    /// 柱の状態遷移のテスト（NotChecked → Extending）
    #[test]
    fn test_pillar_status_transitions() {
        let identifier = WallIdentifier::new();

        // NotChecked → Extending の遷移テスト
        let not_checked =
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None);
        let extending = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );

        // NotChecked 状態の確認
        match not_checked {
            MazePointStatus::Wall(WallType::Pillar(status), _) => {
                assert!(matches!(status, PillarExtendStatus::NotChecked));
            }
            _ => panic!("Expected NotChecked pillar"),
        }

        // Extending 状態の確認
        match extending {
            MazePointStatus::Wall(WallType::Pillar(status), _) => {
                assert!(matches!(status, PillarExtendStatus::Extending));
            }
            _ => panic!("Expected Extending pillar"),
        }
    }

    /// 柱の状態変換チェーンのテスト
    #[test]
    fn test_pillar_state_conversion_chain() {
        let identifier = WallIdentifier::new();

        // NotChecked → Extending の変換チェーン
        let initial_pillar = MazePointStatus::new_notchecked_pillar();

        // 初期状態の確認
        assert!(initial_pillar.is_not_checked_wall());
        assert!(!initial_pillar.is_extending_wall());

        // Extendingに変換
        let extending_result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            initial_pillar,
            &identifier,
        );
        assert!(extending_result.is_ok());

        let extending_pillar = extending_result.unwrap();

        // Extending状態の確認
        assert!(!extending_pillar.is_not_checked_wall());
        assert!(extending_pillar.is_extending_wall());
        assert!(!extending_pillar.is_outside_wall());
        assert!(extending_pillar.is_my_wall(&identifier));

        match extending_pillar {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(id)) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Extending pillar"),
        }
    }

    /// 全ての WallType のパターンマッチングテスト
    #[test]
    fn test_all_wall_types() {
        let outside = WallType::Outside;
        let pillar = WallType::Pillar(PillarExtendStatus::NotChecked);
        let wall = WallType::Wall;

        // 各種類が正しく識別されることを確認
        assert!(matches!(outside, WallType::Outside));
        assert!(matches!(pillar, WallType::Pillar(_)));
        assert!(matches!(wall, WallType::Wall));

        // 他の種類でないことを確認
        assert!(!matches!(outside, WallType::Pillar(_) | WallType::Wall));
        assert!(!matches!(pillar, WallType::Outside | WallType::Wall));
        assert!(!matches!(wall, WallType::Outside | WallType::Pillar(_)));
    }

    /// MazePointStatus の各 Wall バリアントのテスト
    #[test]
    fn test_maze_point_status_wall_variants() {
        let identifier = WallIdentifier::new();

        // 各種壁タイプのMazePointStatusをテスト
        let outside_wall = MazePointStatus::Wall(WallType::Outside, Some(identifier));
        let pillar_wall =
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None);
        let normal_wall = MazePointStatus::Wall(WallType::Wall, Some(identifier));

        // 全てwall判定されることを確認
        assert!(matches!(outside_wall, MazePointStatus::Wall(_, _)));
        assert!(matches!(pillar_wall, MazePointStatus::Wall(_, _)));
        assert!(matches!(normal_wall, MazePointStatus::Wall(_, _)));

        // 全てpath判定されないことを確認
        assert!(!matches!(outside_wall, MazePointStatus::Path));
        assert!(!matches!(pillar_wall, MazePointStatus::Path));
        assert!(!matches!(normal_wall, MazePointStatus::Path));

        // 各wall_typeが正しく取得できることを確認
        assert!(matches!(
            outside_wall,
            MazePointStatus::Wall(WallType::Outside, _)
        ));
        assert!(matches!(
            pillar_wall,
            MazePointStatus::Wall(WallType::Pillar(_), _)
        ));
        assert!(matches!(
            normal_wall,
            MazePointStatus::Wall(WallType::Wall, _)
        ));
    }

    /// WallIdentifier の文字列形式のテスト
    #[test]
    fn test_wall_identifier_format() {
        let identifier = WallIdentifier::new();
        let id_str = identifier.as_str();

        // 形式の確認: "ThreadId(...)" + "_" + 数字
        assert!(id_str.starts_with("ThreadId("));
        assert!(id_str.contains("_"));

        // アンダースコア以降が数字であることを確認
        if let Some(pos) = id_str.rfind('_') {
            let time_part = &id_str[pos + 1..];
            assert!(
                time_part.parse::<i64>().is_ok(),
                "Time part should be a valid number: {}",
                time_part
            );
        } else {
            panic!("Identifier should contain underscore");
        }
    }

    /// MazePointStatus のパターンマッチングテスト
    #[test]
    fn test_maze_point_status_matches() {
        let path = MazePointStatus::Path;
        let identifier = WallIdentifier::new();
        let wall = MazePointStatus::Wall(WallType::Outside, Some(identifier));

        // Path状態のテスト
        match path {
            MazePointStatus::Path => {
                // 正常
            }
            MazePointStatus::Wall(_, _) => {
                panic!("Expected Path status");
            }
        }

        // Wall状態のテスト
        match wall {
            MazePointStatus::Path => {
                panic!("Expected Wall status");
            }
            MazePointStatus::Wall(wall_type, id) => {
                assert!(matches!(wall_type, WallType::Outside));
                assert!(id.is_some());
            }
        }
    }

    /// PillarExtendStatus の全バリアントのテスト
    #[test]
    fn test_pillar_extend_status_all_variants() {
        // 利用可能な全ての状態をテスト
        let not_checked = PillarExtendStatus::NotChecked;
        let extending = PillarExtendStatus::Extending;

        // 各状態が正しく識別されることを確認
        assert!(matches!(not_checked, PillarExtendStatus::NotChecked));
        assert!(matches!(extending, PillarExtendStatus::Extending));

        // 相互排他性を確認
        assert!(!matches!(not_checked, PillarExtendStatus::Extending));
        assert!(!matches!(extending, PillarExtendStatus::NotChecked));
    }

    /// Extending 状態の柱と識別子の関係テスト
    #[test]
    fn test_pillar_extending_with_identifier() {
        let identifier = WallIdentifier::new();

        // Extending状態の柱は識別子を持つ
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );

        match extending_pillar {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(id)) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Extending pillar with identifier"),
        }
    }

    /// 柱の状態と識別子要件のテスト
    #[test]
    fn test_pillar_identifier_requirements() {
        let identifier = WallIdentifier::new();

        // NotChecked柱は識別子を持たない
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        match not_checked_pillar {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None) => {
                // 正常
            }
            _ => panic!("NotChecked pillar should not have identifier"),
        }

        // Extending柱は識別子を持つ
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );
        match extending_pillar {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extending), Some(_)) => {
                // 正常
            }
            _ => panic!("Extending pillar should have identifier"),
        }
    }

    /// is_my_wall メソッドのテスト
    #[test]
    fn test_is_my_wall_method() {
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 外壁のテスト
        let outside_wall = MazePointStatus::new_outside_wall(identifier1);
        assert!(outside_wall.is_my_wall(&identifier1));
        assert!(!outside_wall.is_my_wall(&identifier2));

        // 通路は壁ではない
        let path = MazePointStatus::Path;
        assert!(!path.is_my_wall(&identifier1));

        // 識別子のない柱は誰の壁でもない
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        assert!(!not_checked_pillar.is_my_wall(&identifier1));

        // Extending柱のテスト
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier1),
        );
        assert!(extending_pillar.is_my_wall(&identifier1));
        assert!(!extending_pillar.is_my_wall(&identifier2));
    }

    /// マルチスレッド環境での識別子一意性テスト
    #[test]
    fn test_wall_identifier_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let identifiers = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        // 複数スレッドで識別子を生成
        for _ in 0..5 {
            let identifiers_clone = Arc::clone(&identifiers);
            let handle = thread::spawn(move || {
                let identifier = WallIdentifier::new();

                // 生成された識別子を保存
                identifiers_clone.lock().unwrap().push(identifier);
            });
            handles.push(handle);
        }

        // 全てのスレッドの処理が完了するのを待つ
        for handle in handles {
            handle.join().unwrap();
        }

        // 最終的に全ての識別子が一意であることを確認
        let identifiers_lock = identifiers.lock().unwrap();
        for i in 0..identifiers_lock.len() {
            for j in i + 1..identifiers_lock.len() {
                assert_ne!(identifiers_lock[i].as_str(), identifiers_lock[j].as_str());
            }
        }
    }

    /// 新しい判定メソッドの包括的テスト
    #[test]
    fn test_comprehensive_wall_type_detection() {
        let identifier = WallIdentifier::new();

        // 全ての状態について判定メソッドをテスト
        let test_cases = vec![
            (MazePointStatus::Path, false, false, false, false),
            (
                MazePointStatus::new_outside_wall(identifier),
                true,
                false,
                false,
                true,
            ),
            (
                MazePointStatus::new_notchecked_pillar(),
                false,
                true,
                false,
                false,
            ),
            (
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::Extending),
                    Some(identifier),
                ),
                false,
                false,
                true,
                true,
            ),
            (
                MazePointStatus::Wall(WallType::Wall, Some(identifier)),
                false,
                false,
                false,
                true,
            ),
        ];

        for (
            status,
            expected_outside,
            expected_not_checked,
            expected_extending,
            expected_my_wall,
        ) in test_cases
        {
            assert_eq!(
                status.is_outside_wall(),
                expected_outside,
                "is_outside_wall failed for {:?}",
                status
            );
            assert_eq!(
                status.is_not_checked_wall(),
                expected_not_checked,
                "is_not_checked_wall failed for {:?}",
                status
            );
            assert_eq!(
                status.is_extending_wall(),
                expected_extending,
                "is_extending_wall failed for {:?}",
                status
            );
            assert_eq!(
                status.is_my_wall(&identifier),
                expected_my_wall,
                "is_my_wall failed for {:?}",
                status
            );
        }
    }

    /// パフォーマンステスト：判定メソッドの効率性
    #[test]
    fn test_wall_type_detection_performance() {
        let identifier = WallIdentifier::new();
        let outside_wall = MazePointStatus::new_outside_wall(identifier);
        let not_checked_pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extending),
            Some(identifier),
        );

        // 大量の判定処理を実行（パフォーマンス確認）
        for _ in 0..1000 {
            assert!(outside_wall.is_outside_wall());
            assert!(!outside_wall.is_not_checked_wall());
            assert!(!outside_wall.is_extending_wall());
            assert!(outside_wall.is_my_wall(&identifier));

            assert!(!not_checked_pillar.is_outside_wall());
            assert!(not_checked_pillar.is_not_checked_wall());
            assert!(!not_checked_pillar.is_extending_wall());
            assert!(!not_checked_pillar.is_my_wall(&identifier));

            assert!(!extending_pillar.is_outside_wall());
            assert!(!extending_pillar.is_not_checked_wall());
            assert!(extending_pillar.is_extending_wall());
            assert!(extending_pillar.is_my_wall(&identifier));
        }
    }

    /// エッジケースのテスト
    #[test]
    fn test_edge_cases_for_new_methods() {
        let identifier = WallIdentifier::new();

        // 同じ識別子での複数の壁
        let wall1 = MazePointStatus::new_outside_wall(identifier);
        let wall2 = MazePointStatus::Wall(WallType::Wall, Some(identifier));

        assert!(wall1.is_my_wall(&identifier));
        assert!(wall2.is_my_wall(&identifier));
        assert!(wall1.is_outside_wall());
        assert!(!wall2.is_outside_wall());

        // 識別子なしの壁（通常はNotChecked柱のみ）
        let pillar_without_id = MazePointStatus::new_notchecked_pillar();
        assert!(!pillar_without_id.is_my_wall(&identifier));
        assert!(pillar_without_id.is_not_checked_wall());

        // 異なる識別子
        let different_identifier = WallIdentifier::new();
        assert!(!wall1.is_my_wall(&different_identifier));
        assert!(!wall2.is_my_wall(&different_identifier));
    }

    /// エラーメッセージの詳細性テスト
    #[test]
    fn test_error_message_details() {
        let identifier = WallIdentifier::new();

        // WallType変換のエラーメッセージテスト
        let wall_type_test_cases = vec![
            (WallType::Outside, "Outside"),
            (WallType::Wall, "Wall"),
            (WallType::Pillar(PillarExtendStatus::Extending), "Extending"),
        ];

        for (invalid_input, _expected_type) in wall_type_test_cases {
            let result = WallType::new_extending_pillar_from_notchecked_pillar(invalid_input);
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("expected NotChecked pillar"));
            assert!(error_msg.contains("found"));
        }

        // MazePointStatus変換のエラーメッセージテスト
        let maze_point_test_cases = vec![
            (MazePointStatus::Path, "Path"),
            (MazePointStatus::new_outside_wall(identifier), "Outside"),
            (
                MazePointStatus::Wall(WallType::Wall, Some(identifier)),
                "Wall",
            ),
        ];

        for (invalid_input, _expected_type) in maze_point_test_cases {
            let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
                invalid_input,
                &identifier,
            );
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("expected NotChecked pillar"));
            assert!(error_msg.contains("found"));
        }
    }
}
