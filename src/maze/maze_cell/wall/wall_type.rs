use crate::maze::maze_cell::wall::{
    extend_status::ExtendStatus, outside_wall_type::OutsideWallType,
};

/// メソッド使用の強制を行うための内部構造体
///
/// この構造体は `WallType` の各バリアントに含まれることで、
/// 直接的な列挙体の構築を困難にし、専用のコンストラクタメソッドの
/// 使用を促進します。これにより型安全性を向上させ、意図しない状態の
/// 組み合わせを防止します。
///
/// # 設計意図
///
/// - **直接構築の抑制**: `WallType::Outside(...)` のような直接構築を複雑化
/// - **コンストラクタ強制**: 専用メソッドの使用を促進
/// - **型安全性向上**: 不正な状態組み合わせの防止
/// - **API一貫性**: 統一されたインターフェースの提供
///
/// # 使用方針
///
/// この構造体は内部実装詳細であり、外部から直接使用することは推奨されません。
/// 必ず適切なコンストラクタメソッドを使用してください。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewMethodEnforcer {}

impl NewMethodEnforcer {
    /// 新しいエンフォーサーインスタンスを生成します
    ///
    /// # 戻り値
    ///
    /// 新しい `NewMethodEnforcer` インスタンス
    fn new() -> Self {
        NewMethodEnforcer {}
    }
}

/// 壁の種類を表す列挙型
///
/// 迷路の各座標点における壁の性質を分類し、迷路生成アルゴリズムが
/// 各座標点に対して適切な処理を選択できるようにします。
///
/// # 設計原則
///
/// ## 型安全性の確保
/// 各バリアントに [`NewMethodEnforcer`] が含まれており、直接的な構築を
/// 困難にしています。必ず専用のコンストラクタメソッドを使用することで、
/// 不正な状態組み合わせを防止し、一貫性のあるAPIを提供します。
///
/// ## 壁の分類体系
/// - **Outside**: 迷路の境界を形成する外壁
/// - **Pillar**: 迷路内部の柱（壁生成の起点となる座標点）
/// - **MazeWall**: 柱から拡張されて生成された通常の壁
///
/// ## 識別子管理との統合
/// 各壁タイプは [`WallIdentifier`] との組み合わせで管理され、
/// 壁の所有権とライフサイクルを追跡します：
/// - **Outside**: 種類と状態に応じた識別子管理
/// - **Pillar**: 拡張状態による識別子の有無制御
/// - **MazeWall**: 常時識別子を持つ構造
///
/// # 状態遷移モデル
///
/// ## Outside壁の状態遷移
/// ```text
/// StartPoint(NotChecked) --mark_as_extending--> StartPoint(Extending)
/// JustWall(NotChecked)   --永続的状態--> (変化なし)
/// ```
///
/// ## Pillar の状態遷移
/// ```text
/// Pillar(NotChecked) --mark_as_extending--> Pillar(Extending)
/// ```
///
/// ## MazeWall の特性
/// ```text
/// MazeWall: 状態遷移なし（固定的な壁）
/// ```
///
/// # パフォーマンス特性
///
/// - **メモリ使用量**: 各バリアント 1-3 enum discriminant + data
/// - **比較性能**: O(1) - discriminant比較による高速判定
/// - **クローン性能**: O(1) - 小さなデータ構造の単純コピー
/// - **ハッシュ性能**: O(1) - discriminantベースの効率的ハッシュ
///
/// # スレッドセーフティ
///
/// `WallType` は `Send + Sync` を実装しており、マルチスレッド環境で
/// 安全に共有・転送できます。内部状態は不変であり、競合状態は発生しません。
///
/// # 相互運用性
///
/// ## 関連型との統合
/// - [`MazePointStatus`]: 座標点の状態管理での使用
/// - [`ExtendStatus`]: 拡張処理の状態追跡
/// - [`OutsideWallType`]: 外壁の詳細分類
/// - [`WallIdentifier`]: 壁の所有権管理
///
/// ## エラーハンドリング
/// 不正な状態変換要求に対して、詳細なエラー情報を含む
/// `Box<dyn std::error::Error>` を返します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WallType {
    /// 外壁（迷路の境界）
    ///
    /// 迷路の最外周に配置される壁で、境界形成と迷路生成開始点の
    /// 両方の役割を果たします。
    ///
    /// # 構造仕様
    ///
    /// `Outside(OutsideWallType, ExtendStatus, NewMethodEnforcer)`
    /// - **第1要素**: 外壁の具体的な種類分類（[`OutsideWallType`]）
    /// - **第2要素**: 拡張処理における状態（[`ExtendStatus`]）
    /// - **第3要素**: 型安全性強制器（内部実装詳細）
    ///
    /// # 配置規則
    ///
    /// - **境界座標**: x=0, y=0, x=width-1, y=height-1
    /// - **開始点座標**: 境界上の0以外の偶数座標（x%2==0 && y%2==0）
    /// - **通常外壁**: その他の境界座標
    ///
    /// # 識別子管理方針
    ///
    /// | 外壁種類 | 拡張状態 | 識別子 |
    /// |----------|----------|--------|
    /// | StartPoint | NotChecked | なし |
    /// | StartPoint | Extending | あり |
    /// | JustWall | NotChecked | あり |
    ///
    /// # 処理特性
    ///
    /// - **StartPoint**: 迷路生成アルゴリズムの開始点として機能
    /// - **JustWall**: 固定的な境界として永続的に存在
    /// - **状態変換**: StartPointのみNotChecked→Extending変換可能
    /// - **拡張処理**: StartPoint(Extending)から内部柱への接続を試行
    Outside(OutsideWallType, ExtendStatus, NewMethodEnforcer),

    /// 柱（壁生成の起点）
    ///
    /// 迷路内部に配置される特別な座標点で、壁の拡張処理における
    /// 起点として機能します。
    ///
    /// # 構造仕様
    ///
    /// `Pillar(ExtendStatus, NewMethodEnforcer)`
    /// - **第1要素**: 拡張処理における状態（[`ExtendStatus`]）
    /// - **第2要素**: 型安全性強制器（内部実装詳細）
    ///
    /// # 配置規則
    ///
    /// - **内部座標**: 境界に接しない偶数座標（x%2==0 && y%2==0）
    /// - **座標範囲**: 2 ≤ x ≤ width-3, 2 ≤ y ≤ height-3
    /// - **格子配置**: 規則的な格子パターンで配置
    ///
    /// # 状態遷移
    ///
    /// ```text
    /// NotChecked --拡張処理開始--> Extending
    /// ```
    ///
    /// # 識別子管理方針
    ///
    /// | 拡張状態 | 識別子 | 意味 |
    /// |----------|--------|------|
    /// | NotChecked | なし | 未処理状態 |
    /// | Extending | あり | 処理中状態 |
    ///
    /// # 処理特性
    ///
    /// - **起点機能**: 他の柱や外壁への拡張処理の出発点
    /// - **接続処理**: 距離2の隣接柱との接続を試行
    /// - **中間壁生成**: 接続時に中間点へのMazeWall配置
    /// - **状態管理**: 拡張状態による処理フェーズの制御
    Pillar(ExtendStatus, NewMethodEnforcer),

    /// 生成された壁（迷路の内部構造）
    ///
    /// 柱間または外壁-柱間の拡張処理により中間点に生成される
    /// 通常の壁で、迷路の内部構造を形成します。
    ///
    /// # 構造仕様
    ///
    /// `MazeWall(NewMethodEnforcer)`
    /// - **要素**: 型安全性強制器（内部実装詳細）
    ///
    /// # 配置規則
    ///
    /// - **中間座標**: 拡張処理時の中間点（奇数座標を含む）
    /// - **生成条件**: 2つの拡張可能点間の距離が2の場合
    /// - **座標計算**: (start + end) / 2 の位置に配置
    ///
    /// # 特性
    ///
    /// - **固定性**: 一度生成されると状態変更されない
    /// - **識別子**: 常に生成元スレッドの識別子を持つ
    /// - **構造性**: 迷路の通路と壁の境界を明確化
    /// - **永続性**: 迷路完成まで維持される
    ///
    /// # 識別子管理
    ///
    /// - **必須**: 常に [`WallIdentifier`] と組み合わせて管理
    /// - **目的**: 生成元スレッドの追跡とデバッグ支援
    /// - **一意性**: 各壁が固有の識別子を持つ
    MazeWall(NewMethodEnforcer),
}

impl WallType {
    /// スタートポイント外壁を生成します
    ///
    /// 迷路生成の開始点として使用される外壁を作成します。
    /// 未チェック状態で初期化され、後に拡張処理の起点として機能します。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `Outside(StartPoint, NotChecked, _)`
    /// - **初期状態**: 拡張処理前の待機状態
    /// - **識別子**: 初期状態では未割り当て
    /// - **配置対象**: 境界上の0以外偶数座標
    ///
    /// # 戻り値
    ///
    /// 未チェック状態のスタートポイント外壁
    pub fn new_start_point_outside_wall() -> Self {
        WallType::Outside(
            OutsideWallType::StartPoint,
            ExtendStatus::NotChecked,
            NewMethodEnforcer::new(),
        )
    }

    /// 未チェックスタートポイントを拡張中状態に変換します
    ///
    /// 未チェック状態のスタートポイント外壁を拡張処理中の状態に変更します。
    /// 型安全性を保証するため、入力検証を実施します。
    ///
    /// # 変換規則
    ///
    /// ```text
    /// Outside(StartPoint, NotChecked, _) → Outside(StartPoint, Extending, _)
    /// ```
    ///
    /// # 引数
    ///
    /// * `src_wall` - 変換元の壁タイプ（未チェックスタートポイント限定）
    ///
    /// # 戻り値
    ///
    /// - **成功**: 拡張中状態のスタートポイント
    /// - **失敗**: 詳細なエラー情報
    ///
    /// # エラー条件
    ///
    /// - 入力が `Outside` バリアント以外
    /// - `Outside` だが `StartPoint` 以外
    /// - `StartPoint` だが `NotChecked` 以外の状態
    pub fn new_extending_start_point_from_notchecked_start_point(
        src_wall: WallType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_wall {
            WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::NotChecked, _) => {
                Ok(WallType::Outside(
                    OutsideWallType::StartPoint,
                    ExtendStatus::Extending,
                    NewMethodEnforcer::new(),
                ))
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Cannot convert to extending start point: expected NotChecked start point, found {:?}",
                    src_wall
                ),
            ))),
        }
    }

    /// 通常の外壁を生成します
    ///
    /// 境界を形成する一般的な外壁を作成します。開始点として使用されず、
    /// 固定的な境界として機能します。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `Outside(JustWall, NotChecked, _)`
    /// - **用途**: 迷路の固定境界
    /// - **状態**: 変更されない永続的状態
    /// - **識別子**: 別途管理による境界追跡
    ///
    /// # 戻り値
    ///
    /// 通常の外壁（境界用）
    pub fn new_just_outside_wall() -> Self {
        WallType::Outside(
            OutsideWallType::JustWall,
            ExtendStatus::NotChecked,
            NewMethodEnforcer::new(),
        )
    }

    /// 未チェック状態の柱を生成します
    ///
    /// 迷路内部に配置される初期状態の柱を作成します。
    /// 迷路生成アルゴリズムの処理対象として使用されます。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `Pillar(NotChecked, _)`
    /// - **初期状態**: 拡張処理待ちの状態
    /// - **識別子**: 未割り当て（処理開始時に割り当て）
    /// - **配置対象**: 内部偶数座標
    ///
    /// # 戻り値
    ///
    /// 未チェック状態の柱
    pub fn new_not_checked_pillar() -> Self {
        WallType::Pillar(ExtendStatus::NotChecked, NewMethodEnforcer::new())
    }

    /// 未チェック柱を拡張中状態に変換します
    ///
    /// 未チェック状態の柱を拡張処理中の状態に変更します。
    /// 型安全性を保証するため、厳格な入力検証を実施します。
    ///
    /// # 変換規則
    ///
    /// ```text
    /// Pillar(NotChecked, _) → Pillar(Extending, _)
    /// ```
    ///
    /// # 引数
    ///
    /// * `src_wall` - 変換元の壁タイプ（未チェック柱限定）
    ///
    /// # 戻り値
    ///
    /// - **成功**: 拡張中状態の柱
    /// - **失敗**: 詳細なエラー情報
    ///
    /// # エラー条件
    ///
    /// - 入力が `Pillar` バリアント以外（`Outside` や `MazeWall`）
    /// - `Pillar` だが `NotChecked` 以外の状態
    /// - その他の予期しない状態
    pub fn new_extending_pillar_from_notchecked_pillar(
        src_wall: WallType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_wall {
            WallType::Pillar(ExtendStatus::NotChecked, _) => Ok(WallType::Pillar(
                ExtendStatus::Extending,
                NewMethodEnforcer::new(),
            )),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Cannot convert to extending pillar: expected NotChecked pillar, found {:?}",
                    src_wall
                ),
            ))),
        }
    }

    /// 通常の壁を生成します
    ///
    /// 拡張処理により中間点に生成される通常の壁を作成します。
    /// 迷路の内部構造を形成する主要な要素です。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `MazeWall(_)`
    /// - **配置位置**: 拡張処理時の中間点
    /// - **識別子**: 生成元スレッドによる管理
    /// - **用途**: 迷路内部構造の形成
    ///
    /// # 戻り値
    ///
    /// 通常の壁（迷路内部用）
    pub fn new_maze_wall() -> Self {
        WallType::MazeWall(NewMethodEnforcer::new())
    }

    /// この壁が未チェックのスタートポイントかどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Outside(StartPoint, NotChecked, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 未チェックスタートポイントの場合は `true`
    pub fn is_not_checked_start_point(&self) -> bool {
        matches!(
            self,
            WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::NotChecked, _)
        )
    }

    /// この壁が拡張中のスタートポイントかどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Outside(StartPoint, Extending, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 拡張中スタートポイントの場合は `true`
    pub fn is_extending_start_point(&self) -> bool {
        matches!(
            self,
            WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::Extending, _)
        )
    }

    /// この壁が通常の外壁かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Outside(JustWall, _, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 通常外壁の場合は `true`
    pub fn is_just_wall(&self) -> bool {
        matches!(self, WallType::Outside(OutsideWallType::JustWall, _, _))
    }

    /// この壁が未チェックの柱かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Pillar(NotChecked, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 未チェック柱の場合は `true`
    pub fn is_not_checked_pillar(&self) -> bool {
        matches!(self, WallType::Pillar(ExtendStatus::NotChecked, _))
    }

    /// この壁が拡張中の柱かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Pillar(Extending, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 拡張中柱の場合は `true`
    pub fn is_extending_pillar(&self) -> bool {
        matches!(self, WallType::Pillar(ExtendStatus::Extending, _))
    }

    /// この壁が迷路壁かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `MazeWall(_)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 迷路壁の場合は `true`
    pub fn is_maze_wall(&self) -> bool {
        matches!(self, WallType::MazeWall(_))
    }

    /// この壁が外壁（境界）かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Outside(_, _, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 外壁の場合は `true`
    pub fn is_outside_wall(&self) -> bool {
        matches!(self, WallType::Outside(_, _, _))
    }

    /// この壁が柱（いずれかの状態）かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// `Pillar(_, _)` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 柱の場合は `true`
    pub fn is_pillar(&self) -> bool {
        matches!(self, WallType::Pillar(_, _))
    }

    /// この壁が未チェック状態（いずれかの種類）かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// 拡張状態が `NotChecked` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 未チェック状態の場合は `true`
    pub fn is_not_checked(&self) -> bool {
        match self {
            WallType::Outside(_, ExtendStatus::NotChecked, _) => true,
            WallType::Outside(_, ExtendStatus::Extending, _) => false,
            WallType::Pillar(ExtendStatus::NotChecked, _) => true,
            WallType::Pillar(ExtendStatus::Extending, _) => false,
            WallType::MazeWall(_) => false,
        }
    }

    /// この壁が拡張中状態（いずれかの種類）かどうかを判定します
    ///
    /// # 判定条件
    ///
    /// 拡張状態が `Extending` の場合に `true`
    ///
    /// # 戻り値
    ///
    /// 拡張中状態の場合は `true`
    pub fn is_extending(&self) -> bool {
        match self {
            WallType::Outside(_, ExtendStatus::Extending, _) => true,
            WallType::Outside(_, ExtendStatus::NotChecked, _) => false,
            WallType::Pillar(ExtendStatus::Extending, _) => true,
            WallType::Pillar(ExtendStatus::NotChecked, _) => false,
            WallType::MazeWall(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_method_enforcer() {
        let enforcer1 = NewMethodEnforcer::new();
        let enforcer2 = NewMethodEnforcer::new();

        // 等価性の確認
        assert_eq!(enforcer1, enforcer2);

        // Clone trait の確認
        let cloned = enforcer1.clone();
        assert_eq!(enforcer1, cloned);

        // Debug trait の確認
        let debug_str = format!("{:?}", enforcer1);
        assert!(debug_str.contains("NewMethodEnforcer"));
    }

    #[test]
    fn test_start_point_outside_wall_creation() {
        let start_wall = WallType::new_start_point_outside_wall();

        // 基本的な判定メソッドのテスト
        assert!(start_wall.is_not_checked_start_point());
        assert!(!start_wall.is_extending_start_point());
        assert!(!start_wall.is_just_wall());
        assert!(!start_wall.is_not_checked_pillar());
        assert!(!start_wall.is_extending_pillar());
        assert!(!start_wall.is_maze_wall());

        // 分類メソッドのテスト
        assert!(start_wall.is_outside_wall());
        assert!(!start_wall.is_pillar());
        assert!(start_wall.is_not_checked());
        assert!(!start_wall.is_extending());

        // パターンマッチの確認
        match start_wall {
            WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::NotChecked, _) => {
                // 期待される状態
            }
            _ => panic!("Unexpected wall type: {:?}", start_wall),
        }
    }

    #[test]
    fn test_start_point_state_conversion() {
        let notchecked_start = WallType::new_start_point_outside_wall();

        // 正常な変換
        let extending_start =
            WallType::new_extending_start_point_from_notchecked_start_point(notchecked_start)
                .expect("Should convert successfully");

        // 変換後の状態確認
        assert!(!extending_start.is_not_checked_start_point());
        assert!(extending_start.is_extending_start_point());
        assert!(!extending_start.is_just_wall());
        assert!(extending_start.is_outside_wall());
        assert!(!extending_start.is_not_checked());
        assert!(extending_start.is_extending());

        // パターンマッチの確認
        match extending_start {
            WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::Extending, _) => {
                // 期待される状態
            }
            _ => panic!(
                "Unexpected wall type after conversion: {:?}",
                extending_start
            ),
        }
    }

    #[test]
    fn test_start_point_conversion_error_cases() {
        // 通常外壁からの変換エラー
        let just_wall = WallType::new_just_outside_wall();
        let result = WallType::new_extending_start_point_from_notchecked_start_point(just_wall);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked start point")
        );

        // 柱からの変換エラー
        let pillar = WallType::new_not_checked_pillar();
        let result = WallType::new_extending_start_point_from_notchecked_start_point(pillar);
        assert!(result.is_err());

        // 迷路壁からの変換エラー
        let maze_wall = WallType::new_maze_wall();
        let result = WallType::new_extending_start_point_from_notchecked_start_point(maze_wall);
        assert!(result.is_err());

        // 既に拡張中のスタートポイントからの変換エラー
        let notchecked = WallType::new_start_point_outside_wall();
        let extending =
            WallType::new_extending_start_point_from_notchecked_start_point(notchecked).unwrap();
        let result = WallType::new_extending_start_point_from_notchecked_start_point(extending);
        assert!(result.is_err());
    }

    #[test]
    fn test_just_outside_wall_creation() {
        let just_wall = WallType::new_just_outside_wall();

        // 基本的な判定メソッドのテスト
        assert!(!just_wall.is_not_checked_start_point());
        assert!(!just_wall.is_extending_start_point());
        assert!(just_wall.is_just_wall());
        assert!(!just_wall.is_not_checked_pillar());
        assert!(!just_wall.is_extending_pillar());
        assert!(!just_wall.is_maze_wall());

        // 分類メソッドのテスト
        assert!(just_wall.is_outside_wall());
        assert!(!just_wall.is_pillar());
        assert!(just_wall.is_not_checked());
        assert!(!just_wall.is_extending());

        // パターンマッチの確認
        match just_wall {
            WallType::Outside(OutsideWallType::JustWall, ExtendStatus::NotChecked, _) => {
                // 期待される状態
            }
            _ => panic!("Unexpected wall type: {:?}", just_wall),
        }
    }

    #[test]
    fn test_not_checked_pillar_creation() {
        let pillar = WallType::new_not_checked_pillar();

        // 基本的な判定メソッドのテスト
        assert!(!pillar.is_not_checked_start_point());
        assert!(!pillar.is_extending_start_point());
        assert!(!pillar.is_just_wall());
        assert!(pillar.is_not_checked_pillar());
        assert!(!pillar.is_extending_pillar());
        assert!(!pillar.is_maze_wall());

        // 分類メソッドのテスト
        assert!(!pillar.is_outside_wall());
        assert!(pillar.is_pillar());
        assert!(pillar.is_not_checked());
        assert!(!pillar.is_extending());

        // パターンマッチの確認
        match pillar {
            WallType::Pillar(ExtendStatus::NotChecked, _) => {
                // 期待される状態
            }
            _ => panic!("Unexpected wall type: {:?}", pillar),
        }
    }

    #[test]
    fn test_pillar_state_conversion() {
        let notchecked_pillar = WallType::new_not_checked_pillar();

        // 正常な変換
        let extending_pillar =
            WallType::new_extending_pillar_from_notchecked_pillar(notchecked_pillar)
                .expect("Should convert successfully");

        // 変換後の状態確認
        assert!(!extending_pillar.is_not_checked_start_point());
        assert!(!extending_pillar.is_extending_start_point());
        assert!(!extending_pillar.is_just_wall());
        assert!(!extending_pillar.is_not_checked_pillar());
        assert!(extending_pillar.is_extending_pillar());
        assert!(!extending_pillar.is_maze_wall());

        // 分類メソッドのテスト
        assert!(!extending_pillar.is_outside_wall());
        assert!(extending_pillar.is_pillar());
        assert!(!extending_pillar.is_not_checked());
        assert!(extending_pillar.is_extending());

        // パターンマッチの確認
        match extending_pillar {
            WallType::Pillar(ExtendStatus::Extending, _) => {
                // 期待される状態
            }
            _ => panic!(
                "Unexpected wall type after conversion: {:?}",
                extending_pillar
            ),
        }
    }

    #[test]
    fn test_pillar_conversion_error_cases() {
        // 外壁からの変換エラー
        let start_point = WallType::new_start_point_outside_wall();
        let result = WallType::new_extending_pillar_from_notchecked_pillar(start_point);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected NotChecked pillar")
        );

        // 通常外壁からの変換エラー
        let just_wall = WallType::new_just_outside_wall();
        let result = WallType::new_extending_pillar_from_notchecked_pillar(just_wall);
        assert!(result.is_err());

        // 迷路壁からの変換エラー
        let maze_wall = WallType::new_maze_wall();
        let result = WallType::new_extending_pillar_from_notchecked_pillar(maze_wall);
        assert!(result.is_err());

        // 既に拡張中の柱からの変換エラー
        let notchecked = WallType::new_not_checked_pillar();
        let extending = WallType::new_extending_pillar_from_notchecked_pillar(notchecked).unwrap();
        let result = WallType::new_extending_pillar_from_notchecked_pillar(extending);
        assert!(result.is_err());
    }

    #[test]
    fn test_maze_wall_creation() {
        let maze_wall = WallType::new_maze_wall();

        // 基本的な判定メソッドのテスト
        assert!(!maze_wall.is_not_checked_start_point());
        assert!(!maze_wall.is_extending_start_point());
        assert!(!maze_wall.is_just_wall());
        assert!(!maze_wall.is_not_checked_pillar());
        assert!(!maze_wall.is_extending_pillar());
        assert!(maze_wall.is_maze_wall());

        // 分類メソッドのテスト
        assert!(!maze_wall.is_outside_wall());
        assert!(!maze_wall.is_pillar());
        assert!(!maze_wall.is_not_checked());
        assert!(!maze_wall.is_extending());

        // パターンマッチの確認
        match maze_wall {
            WallType::MazeWall(_) => {
                // 期待される状態
            }
            _ => panic!("Unexpected wall type: {:?}", maze_wall),
        }
    }

    #[test]
    fn test_wall_type_equality() {
        // 同じコンストラクタから生成された壁は等しい
        let start1 = WallType::new_start_point_outside_wall();
        let start2 = WallType::new_start_point_outside_wall();
        assert_eq!(start1, start2);

        let pillar1 = WallType::new_not_checked_pillar();
        let pillar2 = WallType::new_not_checked_pillar();
        assert_eq!(pillar1, pillar2);

        let wall1 = WallType::new_maze_wall();
        let wall2 = WallType::new_maze_wall();
        assert_eq!(wall1, wall2);

        // 異なる種類の壁は等しくない
        let start = WallType::new_start_point_outside_wall();
        let pillar = WallType::new_not_checked_pillar();
        let wall = WallType::new_maze_wall();

        assert_ne!(start, pillar);
        assert_ne!(start, wall);
        assert_ne!(pillar, wall);

        // 状態変換前後は等しくない
        let notchecked = WallType::new_not_checked_pillar();
        let extending =
            WallType::new_extending_pillar_from_notchecked_pillar(notchecked.clone()).unwrap();
        assert_ne!(notchecked, extending);
    }

    #[test]
    fn test_wall_type_clone() {
        let original = WallType::new_start_point_outside_wall();
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert!(original.is_not_checked_start_point());
        assert!(cloned.is_not_checked_start_point());
    }

    #[test]
    fn test_wall_type_debug() {
        let start_wall = WallType::new_start_point_outside_wall();
        let debug_str = format!("{:?}", start_wall);

        assert!(debug_str.contains("Outside"));
        assert!(debug_str.contains("StartPoint"));
        assert!(debug_str.contains("NotChecked"));
        assert!(debug_str.contains("NewMethodEnforcer"));
    }

    #[test]
    fn test_wall_type_hash() {
        use std::collections::{HashMap, HashSet};

        let start_wall = WallType::new_start_point_outside_wall();
        let pillar = WallType::new_not_checked_pillar();
        let maze_wall = WallType::new_maze_wall();

        // HashSet での使用
        let mut wall_set = HashSet::new();
        assert!(wall_set.insert(start_wall.clone()));
        assert!(wall_set.insert(pillar.clone()));
        assert!(wall_set.insert(maze_wall.clone()));
        assert!(!wall_set.insert(start_wall.clone())); // 重複

        assert_eq!(wall_set.len(), 3);

        // HashMap での使用
        let mut wall_map = HashMap::new();
        wall_map.insert(start_wall.clone(), "start");
        wall_map.insert(pillar.clone(), "pillar");
        wall_map.insert(maze_wall.clone(), "wall");

        assert_eq!(wall_map.get(&start_wall), Some(&"start"));
        assert_eq!(wall_map.get(&pillar), Some(&"pillar"));
        assert_eq!(wall_map.get(&maze_wall), Some(&"wall"));
    }

    #[test]
    fn test_comprehensive_state_matrix() {
        // 全ての組み合わせをテストする包括的なマトリックス
        let walls = vec![
            (
                "start_not_checked",
                WallType::new_start_point_outside_wall(),
            ),
            ("just_wall", WallType::new_just_outside_wall()),
            ("pillar_not_checked", WallType::new_not_checked_pillar()),
            ("maze_wall", WallType::new_maze_wall()),
        ];

        // 変換可能な壁を追加
        let start_extending = WallType::new_extending_start_point_from_notchecked_start_point(
            WallType::new_start_point_outside_wall(),
        )
        .unwrap();
        let pillar_extending = WallType::new_extending_pillar_from_notchecked_pillar(
            WallType::new_not_checked_pillar(),
        )
        .unwrap();

        let mut extended_walls = walls;
        extended_walls.push(("start_extending", start_extending));
        extended_walls.push(("pillar_extending", pillar_extending));

        // 各壁タイプの判定メソッドマトリックス
        for (name, wall) in &extended_walls {
            println!("Testing wall: {}", name);

            // 基本的な分類
            let is_outside = wall.is_outside_wall();
            let is_pillar = wall.is_pillar();
            let is_maze_wall = wall.is_maze_wall();

            // 状態分類
            let is_not_checked = wall.is_not_checked();
            let is_extending = wall.is_extending();

            // 詳細分類
            let is_not_checked_start = wall.is_not_checked_start_point();
            let is_extending_start = wall.is_extending_start_point();
            let is_just_wall = wall.is_just_wall();
            let is_not_checked_pillar = wall.is_not_checked_pillar();
            let is_extending_pillar = wall.is_extending_pillar();

            // 排他性の確認
            let type_count = [is_outside, is_pillar, is_maze_wall]
                .iter()
                .filter(|&&x| x)
                .count();
            assert_eq!(
                type_count, 1,
                "Wall should belong to exactly one major type: {}",
                name
            );

            // 状態の一貫性確認
            if is_maze_wall {
                assert!(!is_not_checked);
                assert!(!is_extending);
            }

            // 詳細分類の一貫性確認
            if is_outside {
                assert!(is_not_checked_start || is_extending_start || is_just_wall);
                assert!(!is_not_checked_pillar);
                assert!(!is_extending_pillar);
            }

            if is_pillar {
                assert!(!is_not_checked_start);
                assert!(!is_extending_start);
                assert!(!is_just_wall);
                assert!(is_not_checked_pillar || is_extending_pillar);
            }
        }
    }

    #[test]
    fn test_error_message_quality() {
        // エラーメッセージの品質確認
        let start_wall = WallType::new_start_point_outside_wall();
        let pillar = WallType::new_not_checked_pillar();
        let maze_wall = WallType::new_maze_wall();

        // スタートポイント変換でのエラーメッセージ
        let error = WallType::new_extending_start_point_from_notchecked_start_point(pillar.clone())
            .unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("expected NotChecked start point"));
        assert!(error_msg.contains("found"));

        // 柱変換でのエラーメッセージ
        let error =
            WallType::new_extending_pillar_from_notchecked_pillar(maze_wall.clone()).unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("expected NotChecked pillar"));
        assert!(error_msg.contains("found"));

        // 詳細な診断情報の確認
        println!("Error message for pillar conversion: {}", error_msg);
        assert!(error_msg.len() > 20); // 十分な詳細があることを確認
    }

    #[test]
    fn test_state_transition_completeness() {
        // 状態遷移の完全性テスト

        // スタートポイントの遷移チェーン
        let start_initial = WallType::new_start_point_outside_wall();
        assert!(start_initial.is_not_checked_start_point());

        let start_extending =
            WallType::new_extending_start_point_from_notchecked_start_point(start_initial)
                .expect("Should convert start point");
        assert!(start_extending.is_extending_start_point());
        assert!(!start_extending.is_not_checked_start_point());

        // 柱の遷移チェーン
        let pillar_initial = WallType::new_not_checked_pillar();
        assert!(pillar_initial.is_not_checked_pillar());

        let pillar_extending =
            WallType::new_extending_pillar_from_notchecked_pillar(pillar_initial)
                .expect("Should convert pillar");
        assert!(pillar_extending.is_extending_pillar());
        assert!(!pillar_extending.is_not_checked_pillar());

        // 逆方向の遷移は存在しないことを確認
        // （現在の設計では拡張中から未チェックに戻る機能はない）

        // 迷路壁は状態遷移しない
        let maze_wall = WallType::new_maze_wall();
        assert!(maze_wall.is_maze_wall());
        // 迷路壁からの変換は定義されていない
    }

    #[test]
    fn test_memory_layout_consistency() {
        // メモリレイアウトの一貫性確認
        use std::mem;

        let start_wall = WallType::new_start_point_outside_wall();
        let pillar = WallType::new_not_checked_pillar();
        let maze_wall = WallType::new_maze_wall();

        // 全ての variant が同じサイズであることを確認
        assert_eq!(mem::size_of_val(&start_wall), mem::size_of::<WallType>());
        assert_eq!(mem::size_of_val(&pillar), mem::size_of::<WallType>());
        assert_eq!(mem::size_of_val(&maze_wall), mem::size_of::<WallType>());

        // discriminant の確認
        assert_ne!(mem::discriminant(&start_wall), mem::discriminant(&pillar));
        assert_ne!(
            mem::discriminant(&start_wall),
            mem::discriminant(&maze_wall)
        );
        assert_ne!(mem::discriminant(&pillar), mem::discriminant(&maze_wall));

        println!("WallType size: {} bytes", mem::size_of::<WallType>());
        println!(
            "NewMethodEnforcer size: {} bytes",
            mem::size_of::<NewMethodEnforcer>()
        );
    }
}
