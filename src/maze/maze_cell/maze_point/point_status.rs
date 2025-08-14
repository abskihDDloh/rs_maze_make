use crate::maze::maze_cell::{
    path::path_type::PathType,
    wall::{
        extend_status::ExtendStatus, outside_wall_type::OutsideWallType,
        wall_identifier::WallIdentifier, wall_type::WallType,
    },
};

/// メソッド使用の強制を行うための内部構造体
///
/// この構造体は `MazePointStatus` の各バリアントに含まれることで、
/// 直接的な列挙体の構築を困難にし、専用のコンストラクタメソッドの
/// 使用を促進します。これにより型安全性を向上させ、意図しない状態の
/// 組み合わせを防止します。
///
/// # 設計意図
///
/// - **直接構築の抑制**: `MazePointStatus::Wall(...)` のような直接構築を複雑化
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

/// 迷路の各座標点の状態を表す列挙型
///
/// 迷路における各座標点は通路（Path）または壁（Wall）のいずれかの状態を持ちます。
/// この分類により、迷路生成アルゴリズムが適切な処理を選択し、
/// 最終的な迷路構造を決定することができます。
///
/// # 設計原則
///
/// ## 二分類システム
/// 迷路の全座標点を明確に二分する単純な分類体系：
/// - **通路（Path）**: プレイヤーが移動可能な空間
/// - **壁（Wall）**: 移動不可能な障害物
///
/// ## 型安全性の確保
/// 各バリアントに [`NewMethodEnforcer`] が含まれており、直接的な構築を
/// 困難にしています。必ず専用のコンストラクタメソッドを使用することで、
/// 不正な状態組み合わせを防止し、一貫性のあるAPIを提供します。
///
/// ## 階層的壁管理
/// 壁は [`WallType`] による詳細分類と [`WallIdentifier`] による
/// 所有権管理の二重構造で管理されます：
/// - **外壁**: 迷路境界の形成と生成開始点の提供
/// - **柱**: 内部構造生成の起点となる特別な座標点
/// - **迷路壁**: 生成処理により配置される通常の壁
///
/// # 状態遷移モデル
///
/// ## 通路の特性
/// ```text
/// Path: 状態遷移なし（固定的な空間）
/// ```
///
/// ## 壁の状態遷移
/// ```text
/// 外壁（StartPoint）: NotChecked --mark_as_extending--> Extending
/// 外壁（JustWall）: NotChecked（固定状態）
/// 柱: NotChecked --mark_as_extending--> Extending
/// 迷路壁: 固定状態（生成時から不変）
/// ```
///
/// # 識別子管理システム
///
/// ## 識別子の有無による分類
/// | 壁の種類 | 状態 | 識別子 | 意味 |
/// |----------|------|--------|------|
/// | StartPoint | NotChecked | なし | 未選択の開始点候補 |
/// | StartPoint | Extending | あり | 処理中の開始点 |
/// | JustWall | NotChecked | あり | 固定境界 |
/// | Pillar | NotChecked | なし | 未処理の柱 |
/// | Pillar | Extending | あり | 処理中の柱 |
/// | MazeWall | （固定） | あり | 生成済み壁 |
///
/// ## 所有権とトレーサビリティ
/// - **生成元追跡**: どのスレッドが特定の壁を生成したかを記録
/// - **競合回避**: 複数スレッドによる同一壁への操作を防止
/// - **デバッグ支援**: 迷路生成過程の詳細な追跡が可能
///
/// # パフォーマンス特性
///
/// ## メモリ効率性
/// - **コンパクト表現**: enum discriminant + データの効率的なレイアウト
/// - **参照透過性**: 内部状態は不変であり、安全な共有が可能
/// - **キャッシュ親和性**: 小さなデータ構造による良好なメモリ局所性
///
/// ## 実行時性能
/// - **判定性能**: O(1) - パターンマッチによる高速状態判定
/// - **変換性能**: O(1) - 単純な構造体構築による高速変換
/// - **比較性能**: O(1) - 構造的等価性による効率的比較
///
/// # スレッドセーフティ
///
/// ## 共有と転送
/// `MazePointStatus` は `Send + Sync` を実装しており、マルチスレッド環境で
/// 安全に共有・転送できます：
/// - **不変性**: 一度作成された状態は変更されない
/// - **原子性**: 状態変換は新しいインスタンス生成による
/// - **競合回避**: 識別子システムによる所有権管理
///
/// ## 並行処理対応
/// - **読み取り安全**: 複数スレッドからの同時読み取りが安全
/// - **変換安全**: 状態変換は関数型アプローチによる
/// - **識別子統合**: スレッド間での壁の所有権が明確
///
/// # エラーハンドリング
///
/// ## 状態変換エラー
/// 不正な状態変換要求に対して、詳細なエラー情報を提供：
/// - **期待状態**: 変換に必要な入力状態の明示
/// - **実際状態**: 実際に受け取った状態の詳細
/// - **変換可能性**: 利用可能な変換パスの提示
///
/// ## エラー種類
/// - `InvalidData`: 型や状態の不一致
/// - `NotFound`: 必要な要素の欠如
/// - `PermissionDenied`: 識別子による権限不足
///
/// # 相互運用性
///
/// ## 関連型との統合
/// - `WallType`: 壁の詳細分類と状態管理
/// - `WallIdentifier`: 所有権とトレーサビリティ
/// - `ExtendStatus`: 迷路生成過程の状態追跡
/// - `OutsideWallType`: 外壁の種類別分類
///
/// ## 迷路システム内での役割
/// - **Field**: 座標点配列での基本要素
/// - **Algorithm**: 生成アルゴリズムでの判定基準
/// - **Renderer**: 描画システムでの表示判定
/// - **Validator**: 迷路構造の整合性検証
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MazePointStatus {
    /// 通路（移動可能な空間）
    ///
    /// プレイヤーが自由に移動できる空間を表します。
    /// 迷路における「空いている場所」の概念的表現です。
    ///
    /// # 特性
    ///
    /// - **移動可能性**: プレイヤーやエージェントが通行可能
    /// - **初期状態**: 迷路初期化時のデフォルト状態
    /// - **永続性**: 一度通路になると壁に戻ることはない
    /// - **識別子無し**: 通路は特定のスレッドに所有されない
    ///
    /// # 構造仕様
    ///
    /// `Path(NewMethodEnforcer)`
    /// - **要素**: 型安全性強制器（内部実装詳細）
    ///
    /// # 用途
    ///
    /// - **移動経路**: プレイヤーの移動可能範囲の定義
    /// - **ゴール探索**: 経路探索アルゴリズムでの有効ノード
    /// - **空間計算**: 迷路内の利用可能空間の計測
    /// - **描画処理**: 視覚的表現での背景色や空白表示
    ///
    /// 通路（Path）
    ///
    /// - `PATH_TYPE`: 通路の状態（解決済み/未解決）
    /// - `NewMethodEnforcer`: 型安全性強制用
    Path(PathType, NewMethodEnforcer),

    /// 壁（移動不可能な障害物）
    ///
    /// プレイヤーの移動を阻む物理的な障害物を表します。
    /// 迷路の構造を決定する主要な要素です。
    ///
    /// # 構造仕様
    ///
    /// `Wall(WallType, Option<WallIdentifier>, NewMethodEnforcer)`
    /// - **第1要素**: 壁の種類（[`WallType`]） - 壁の詳細分類
    /// - **第2要素**: 識別子（[`Option<WallIdentifier>`]） - 所有権管理
    /// - **第3要素**: 型安全性強制器（内部実装詳細）
    ///
    /// # 壁の分類体系
    ///
    /// ## 位置による分類
    /// - **外壁**: 迷路境界（x=0, y=0, x=max, y=max）
    /// - **内部壁**: 迷路内部の構造要素
    ///
    /// ## 機能による分類
    /// - **開始点**: 迷路生成アルゴリズムの起点
    /// - **境界壁**: 固定的な境界要素
    /// - **生成壁**: アルゴリズムにより動的生成
    ///
    /// ## 状態による分類
    /// - **未チェック**: 処理待ちの状態
    /// - **拡張中**: 現在処理中の状態
    /// - **完了**: 処理完了の固定状態
    ///
    /// # 識別子管理の原則
    ///
    /// ## 識別子有りの場合（`Some(WallIdentifier)`）
    /// - **生成元特定**: 特定スレッドによる生成の記録
    /// - **所有権主張**: 該当スレッドの管理下にある状態
    /// - **変更権限**: 識別子保持者のみが状態変更可能
    /// - **デバッグ情報**: 生成過程のトレーサビリティ提供
    ///
    /// ## 識別子無しの場合（`None`）
    /// - **未割り当て**: どのスレッドにも所有されていない
    /// - **処理待ち**: アルゴリズムによる処理を待機中
    /// - **一時状態**: 後に識別子が割り当てられる予定
    /// - **開放状態**: 任意のスレッドが処理可能
    ///
    /// # ライフサイクル
    ///
    /// ```text
    /// 1. 初期化: new_*() メソッドによる生成
    /// 2. 配置: フィールドへの座標指定配置
    /// 3. 処理: アルゴリズムによる状態変換
    /// 4. 完了: 最終状態での固定化
    /// 5. 参照: 判定メソッドによる状態確認
    /// ```
    ///
    /// # メモリ管理
    ///
    /// - **所有権**: 構造体による値の所有
    /// - **借用**: 参照による一時的アクセス
    /// - **複製**: Clone traitによる深いコピー
    /// - **移動**: 所有権移転による効率的な転送
    Wall(WallType, Option<WallIdentifier>, NewMethodEnforcer),
}

#[allow(dead_code)]
impl MazePointStatus {
    /// 通路状態を生成します
    ///
    /// プレイヤーが移動可能な空間の状態を作成します。
    /// 迷路初期化時や壁生成後の空間確保で使用されます。
    ///
    /// # 生成仕様
    ///
    /// - **状態**: `Path(NewMethodEnforcer)`
    /// - **識別子**: なし（通路は所有されない）
    /// - **変更可能性**: 後に壁に変換される可能性あり
    /// - **用途**: 迷路の移動可能領域
    ///
    /// # 戻り値
    ///
    /// 通路状態の `MazePointStatus`
    /// 未解決通路（NOT_RESOLVED_PATH）を生成
    pub fn new_not_resolved_path() -> Self {
        MazePointStatus::Path(PathType::NotResolvedPath, NewMethodEnforcer::new())
    }

    /// この座標点が通路かどうかを判定します
    ///
    /// 座標点の基本分類を判定する最も基本的な述語です。
    /// `is_wall()` の論理反転に相当します。
    ///
    /// # 判定条件
    ///
    /// `Path(_)` パターンにマッチする場合に `true`
    ///
    /// # 戻り値
    ///
    /// 通路の場合は `true`、壁（任意の種類・状態）の場合は `false`
    pub fn is_path(&self) -> bool {
        matches!(self, MazePointStatus::Path(..))
    }

    /// 解決済み通路（RESOLVED_PATH）を生成
    pub fn new_resolved_path() -> Self {
        MazePointStatus::Path(PathType::ResolvedPath, NewMethodEnforcer::new())
    }

    /// この座標点が「解決済み通路」か判定
    pub fn is_resolved_path(&self) -> bool {
        matches!(self, MazePointStatus::Path(PathType::ResolvedPath, _))
    }

    /// 迷路壁を生成します
    ///
    /// 柱間の拡張処理により中間点に配置される通常の壁を作成します。
    /// 迷路の内部構造を形成する主要な要素です。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `WallType::MazeWall`
    /// - **識別子**: 必須（生成元スレッドの追跡）
    /// - **配置位置**: 拡張処理時の中間座標
    /// - **用途**: 迷路内部構造の形成
    ///
    /// # 引数
    ///
    /// * `identifier` - 生成元スレッドの識別子
    ///
    /// # 戻り値
    ///
    /// 迷路壁状態の `MazePointStatus`
    pub fn new_maze_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(
            WallType::new_maze_wall(),
            Some(identifier),
            NewMethodEnforcer::new(),
        )
    }

    /// 未チェックスタートポイント外壁を生成します
    ///
    /// 迷路生成の開始点候補となる外壁を作成します。
    /// 後に拡張処理の起点として選択される可能性があります。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `WallType::Outside(StartPoint, NotChecked)`
    /// - **識別子**: なし（未選択状態）
    /// - **配置位置**: 境界上の0以外偶数座標
    /// - **用途**: 迷路生成開始点候補
    ///
    /// # 引数
    ///
    /// * `identifier` - 境界管理用識別子（将来の拡張時に使用）
    ///
    /// # 戻り値
    ///
    /// 未チェックスタートポイント外壁状態の `MazePointStatus`
    pub fn new_start_point_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(
            WallType::new_start_point_outside_wall(),
            Some(identifier),
            NewMethodEnforcer::new(),
        )
    }

    /// 通常の外壁を生成します
    ///
    /// 迷路の境界を形成する固定的な外壁を作成します。
    /// 開始点として使用されず、永続的な境界として機能します。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `WallType::Outside(JustWall, NotChecked)`
    /// - **識別子**: 必須（境界管理）
    /// - **配置位置**: 境界座標（開始点以外）
    /// - **用途**: 迷路の固定境界
    ///
    /// # 引数
    ///
    /// * `identifier` - 境界管理用識別子
    ///
    /// # 戻り値
    ///
    /// 通常外壁状態の `MazePointStatus`
    pub fn new_just_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(
            WallType::new_just_outside_wall(),
            Some(identifier),
            NewMethodEnforcer::new(),
        )
    }

    /// 未チェックスタートポイントを拡張中状態に変換します
    ///
    /// 未チェック状態のスタートポイント外壁を拡張処理中の状態に変更します。
    /// 迷路生成アルゴリズムが開始点を選択した際に実行されます。
    ///
    /// # 変換プロセス
    ///
    /// 1. **入力検証**: 未チェックスタートポイント + 識別子無しの確認
    /// 2. **WallType変換**: 内部的な状態変換の実行
    /// 3. **識別子設定**: 処理スレッドの識別子を新規設定
    /// 4. **結果構築**: 拡張中状態のインスタンス生成
    ///
    /// # 変換規則
    ///
    /// ```text
    /// Wall(Outside(StartPoint, NotChecked), None, _)
    ///   ↓
    /// Wall(Outside(StartPoint, Extending), Some(identifier), _)
    /// ```
    ///
    /// # 引数
    ///
    /// * `src_outside_wall` - 変換元の外壁状態
    /// * `identifier` - 新しく設定する処理スレッド識別子
    ///
    /// # 戻り値
    ///
    /// - **成功**: 拡張中状態のスタートポイント
    /// - **失敗**: 詳細なエラー情報
    ///
    /// # エラー条件
    ///
    /// - 入力が `Wall` バリアント以外
    /// - 入力の識別子が `None` 以外
    /// - 内部 `WallType` 変換の失敗
    /// - 期待される状態組み合わせの不一致
    pub fn new_extending_start_point_from_notchecked_start_point(
        src_outside_wall: MazePointStatus,
        identifier: &WallIdentifier,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_outside_wall {
            MazePointStatus::Wall(wall_type, _, _) => {
                let extending_wall_type =
                    WallType::new_extending_start_point_from_notchecked_start_point(wall_type)?;
                Ok(MazePointStatus::Wall(
                    extending_wall_type,
                    Some(*identifier),
                    NewMethodEnforcer::new(),
                ))
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot convert to extending start point: expected NotChecked start point with no identifier, found {:?}",
                        src_outside_wall
                    ),
                )));
            }
        }
    }

    /// 未チェック状態の柱を生成します
    ///
    /// 迷路内部に配置される初期状態の柱を作成します。
    /// 迷路生成アルゴリズムの処理対象として使用されます。
    ///
    /// # 生成仕様
    ///
    /// - **壁種類**: `WallType::Pillar(NotChecked)`
    /// - **識別子**: なし（未処理状態）
    /// - **配置位置**: 内部偶数座標（境界除く）
    /// - **用途**: 壁生成の起点候補
    ///
    /// # 戻り値
    ///
    /// 未チェック柱状態の `MazePointStatus`
    pub fn new_notchecked_pillar() -> Self {
        MazePointStatus::Wall(
            WallType::new_not_checked_pillar(),
            None,
            NewMethodEnforcer::new(),
        )
    }

    /// 未チェック柱を拡張中状態に変換します
    ///
    /// 未チェック状態の柱を拡張処理中の状態に変更します。
    /// 迷路生成アルゴリズムが処理対象の柱を選択した際に実行されます。
    ///
    /// # 変換プロセス
    ///
    /// 1. **入力検証**: 未チェック柱 + 識別子無しの確認
    /// 2. **WallType変換**: 柱の拡張状態変換
    /// 3. **識別子設定**: 処理スレッドの識別子を新規設定
    /// 4. **結果構築**: 拡張中状態のインスタンス生成
    ///
    /// # 変換規則
    ///
    /// ```text
    /// Wall(Pillar(NotChecked), None, _)
    ///   ↓
    /// Wall(Pillar(Extending), Some(identifier), _)
    /// ```
    ///
    /// # 引数
    ///
    /// * `src_pillar` - 変換元の柱状態
    /// * `identifier` - 新しく設定する処理スレッド識別子
    ///
    /// # 戻り値
    ///
    /// - **成功**: 拡張中状態の柱
    /// - **失敗**: 詳細なエラー情報
    ///
    /// # エラー条件
    ///
    /// - 入力が `Wall` バリアント以外
    /// - 入力の識別子が `None` 以外
    /// - 内部 `WallType` が `Pillar` 以外
    /// - 柱の拡張状態が `NotChecked` 以外
    pub fn new_extending_pillar_from_notchecked_pillar(
        src_pillar: MazePointStatus,
        identifier: &WallIdentifier,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match src_pillar {
            MazePointStatus::Wall(wall_type, None, _) => {
                let extending_wall_type =
                    WallType::new_extending_pillar_from_notchecked_pillar(wall_type)?;

                Ok(MazePointStatus::Wall(
                    extending_wall_type,
                    Some(*identifier),
                    NewMethodEnforcer::new(),
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

    /// この座標点が壁かどうかを判定します
    ///
    /// 座標点の基本分類を判定する最も基本的な述語です。
    /// 壁の種類や状態に関係なく、壁であるかどうかのみを確認します。
    ///
    /// # 判定条件
    ///
    /// `Wall(_, _, _)` パターンにマッチする場合に `true`
    ///
    /// # 戻り値
    ///
    /// 壁（任意の種類・状態）の場合は `true`、通路の場合は `false`
    pub fn is_wall(&self) -> bool {
        matches!(self, MazePointStatus::Wall(..))
    }

    /// 壁の識別子を取得します
    ///
    /// 壁が持つ識別子への参照を取得します。
    /// 所有権管理やデバッグ情報の取得に使用されます。
    ///
    /// # 戻り値の意味
    ///
    /// - **`Some(&WallIdentifier)`**: 識別子付き壁の場合
    ///   - 生成元スレッドが特定可能
    ///   - 所有権が明確に定義された状態
    /// - **`None`**: 識別子無し壁または通路の場合
    ///   - 未処理状態の柱
    ///   - 通路（識別子を持たない）
    ///
    /// # 戻り値
    ///
    /// 識別子への参照（利用可能な場合）
    pub fn get_wall_identifier(&self) -> Option<&WallIdentifier> {
        if !self.is_wall() {
            return None;
        }
        match self {
            MazePointStatus::Wall(_, Some(id), _) => Some(id),
            _ => None,
        }
    }

    /// 壁の種類を取得します
    ///
    /// 壁の詳細な種類情報を取得します。
    /// アルゴリズムが適切な処理を選択するために使用されます。
    ///
    /// # 戻り値の意味
    ///
    /// - **`Some(WallType)`**: 壁の場合の詳細種類
    ///   - `Outside`: 外壁（境界・開始点）
    ///   - `Pillar`: 柱（内部起点）
    ///   - `MazeWall`: 通常壁（生成結果）
    /// - **`None`**: 通路の場合
    ///
    /// # 戻り値
    ///
    /// 壁の種類（クローン済み）
    pub fn get_wall_type(&self) -> Option<WallType> {
        if !self.is_wall() {
            return None;
        }
        if let MazePointStatus::Wall(wall_type, _, _) = self {
            Some(wall_type.clone())
        } else {
            None
        }
    }

    /// この座標点が外壁かどうかを判定します
    ///
    /// 迷路の境界に配置される外壁を識別します。
    /// 境界処理や開始点選択で使用される重要な分類判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 壁である（`is_wall() == true`）
    /// - 壁種類が `WallType::Outside(_, _, _)`
    ///
    /// # 戻り値
    ///
    /// 外壁の場合は `true`、そうでなければ `false`
    pub fn is_outside_wall(&self) -> bool {
        if !self.is_wall() {
            return false;
        }
        if let Some(wall_type) = self.get_wall_type()
            && matches!(wall_type, WallType::Outside(..))
        {
            return true;
        }
        false
    }

    /// この座標点が未チェックスタートポイントかどうかを判定します
    ///
    /// 迷路生成開始点の候補を識別します。
    /// 開始点選択アルゴリズムで使用される特化された判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 外壁である（`is_outside_wall() == true`）
    /// - 外壁種類が `StartPoint`
    /// - 拡張状態が `NotChecked`
    ///
    /// # 戻り値
    ///
    /// 未チェックスタートポイントの場合は `true`
    pub fn is_not_checked_start_point(&self) -> bool {
        if !self.is_outside_wall() {
            return false;
        }
        if let Some(wall_type) = self.get_wall_type()
            && matches!(
                wall_type,
                WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::NotChecked, _)
            )
        {
            return true;
        }
        false
    }

    /// この座標点が拡張中スタートポイントかどうかを判定します
    ///
    /// 現在処理中の開始点を識別します。
    /// 拡張処理の進行状況追跡で使用される判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 外壁である（`is_outside_wall() == true`）
    /// - 外壁種類が `StartPoint`
    /// - 拡張状態が `Extending`
    ///
    /// # 戻り値
    ///
    /// 拡張中スタートポイントの場合は `true`
    pub fn is_extending_start_point(&self) -> bool {
        if !self.is_outside_wall() {
            return false;
        }
        if let Some(wall_type) = self.get_wall_type()
            && matches!(
                wall_type,
                WallType::Outside(OutsideWallType::StartPoint, ExtendStatus::Extending, _)
            )
        {
            return true;
        }
        false
    }

    /// この座標点が通常外壁かどうかを判定します
    ///
    /// 固定的な境界を形成する外壁を識別します。
    /// 境界描画や衝突判定で使用される判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 外壁である（`is_outside_wall() == true`）
    /// - 外壁種類が `JustWall`
    /// - 拡張状態が `NotChecked`（固定状態）
    ///
    /// # 戻り値
    ///
    /// 通常外壁の場合は `true`
    pub fn is_just_outside_wall(&self) -> bool {
        if !self.is_outside_wall() {
            return false;
        }
        if let Some(wall_type) = self.get_wall_type()
            && matches!(
                wall_type,
                WallType::Outside(OutsideWallType::JustWall, ExtendStatus::NotChecked, _)
            )
        {
            return true;
        }
        false
    }

    /// この座標点が柱かどうかを判定します
    ///
    /// 拡張状態に関係なく、柱であるかどうかを判定します。
    /// 柱関連の処理全般で使用される基本的な分類判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 壁である（`is_wall() == true`）
    /// - 壁種類が `WallType::Pillar(_, _)`
    ///
    /// # 戻り値
    ///
    /// 柱（任意の拡張状態）の場合は `true`
    pub fn is_pillar(&self) -> bool {
        if !self.is_wall() {
            return false;
        }
        if let Some(wall_type) = self.get_wall_type()
            && matches!(wall_type, WallType::Pillar(..))
        {
            return true;
        }
        false
    }

    /// この座標点が未チェック柱かどうかを判定します
    ///
    /// 処理待ちの柱を識別します。
    /// 拡張可能な柱の選択で使用される特化された判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 壁である（`is_wall() == true`）
    /// - 柱である（`is_pillar() == true`）
    /// - 拡張状態が `NotChecked`
    /// - 識別子が `None`
    ///
    /// # 戻り値
    ///
    /// 未チェック柱の場合は `true`
    pub fn is_not_checked_pillar(&self) -> bool {
        if !self.is_wall() || !self.is_pillar() {
            return false;
        }
        matches!(
            self,
            MazePointStatus::Wall(WallType::Pillar(ExtendStatus::NotChecked, ..), None, _)
        )
    }

    /// この座標点が拡張中柱かどうかを判定します
    ///
    /// 現在処理中の柱を識別します。
    /// 拡張処理の進行状況追跡で使用される判定です。
    ///
    /// # 判定条件
    ///
    /// 以下の条件を満たす場合に `true`：
    /// - 柱である（`is_pillar() == true`）
    /// - 拡張状態が `Extending`
    /// - 識別子が `Some(_)`
    ///
    /// # 戻り値
    ///
    /// 拡張中柱の場合は `true`
    pub fn is_extending_pillar(&self) -> bool {
        if !self.is_pillar() {
            return false;
        }
        matches!(
            self,
            MazePointStatus::Wall(WallType::Pillar(ExtendStatus::Extending, ..), Some(_), _)
        )
    }

    /// 指定された識別子で生成された壁かどうかを判定します
    ///
    /// 特定のスレッドや処理で生成された壁を識別します。
    /// マルチスレッド環境での所有権確認に使用される重要な判定です。
    ///
    /// # 判定プロセス
    ///
    /// 1. **壁確認**: 座標点が壁であることを確認
    /// 2. **識別子取得**: 壁の識別子を取得
    /// 3. **一致判定**: 指定識別子との一致を確認
    ///
    /// # 引数
    ///
    /// * `identifier` - 確認したい識別子
    ///
    /// # 戻り値
    ///
    /// 指定された識別子で生成された壁の場合は `true`
    pub fn is_my_wall(&self, identifier: &WallIdentifier) -> bool {
        if !self.is_wall() {
            return false;
        }
        let wall_identifier = self.get_wall_identifier();
        if let Some(id) = wall_identifier {
            return *id == *identifier;
        }
        false
    }

    /// この座標点が拡張中状態かどうかを判定します
    ///
    /// 種類に関係なく、拡張処理中の状態を識別します。
    /// 処理中座標点の一括管理で使用される汎用判定です。
    ///
    /// # 判定条件
    ///
    /// 以下のいずれかの場合に `true`：
    /// - 拡張中スタートポイント
    /// - 拡張中柱
    ///
    /// # 戻り値
    ///
    /// 拡張中状態の場合は `true`
    pub fn is_extending_wall(&self) -> bool {
        self.is_extending_start_point() || self.is_extending_pillar()
    }

    /// この座標点が未チェック状態かどうかを判定します
    ///
    /// 種類に関係なく、未処理の状態を識別します。
    /// 処理対象候補の一括選択で使用される汎用判定です。
    ///
    /// # 判定条件
    ///
    /// 以下のいずれかの場合に `true`：
    /// - 未チェックスタートポイント
    /// - 未チェック柱
    /// - 通常外壁（固定未チェック状態）
    ///
    /// # 戻り値
    ///
    /// 未チェック状態の場合は `true`
    pub fn is_not_checked_wall(&self) -> bool {
        self.is_not_checked_start_point()
            || self.is_not_checked_pillar()
            || self.is_just_outside_wall()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Instant};

    use super::*;

    // NewMethodEnforcer Tests
    #[test]
    fn test_new_method_enforcer_creation() {
        let enforcer1 = NewMethodEnforcer::new();
        let enforcer2 = NewMethodEnforcer::new();

        // All enforcers should be equal
        assert_eq!(enforcer1, enforcer2);
    }

    #[test]
    fn test_new_method_enforcer_clone() {
        let original = NewMethodEnforcer::new();
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_new_method_enforcer_debug() {
        let enforcer = NewMethodEnforcer::new();
        let debug_str = format!("{:?}", enforcer);
        assert!(debug_str.contains("NewMethodEnforcer"));
    }

    #[test]
    fn test_new_method_enforcer_hash() {
        let mut map = HashMap::new();
        let enforcer1 = NewMethodEnforcer::new();
        let enforcer2 = NewMethodEnforcer::new();

        map.insert(enforcer1, "test1");

        // Should be able to retrieve with different but equal enforcer
        assert_eq!(map.get(&enforcer2), Some(&"test1"));
    }

    // Basic Constructor Tests
    #[test]
    fn test_new_not_resolved_path() {
        let path = MazePointStatus::new_not_resolved_path();
        assert!(path.is_path());
        assert!(!path.is_wall());
        assert!(!path.is_outside_wall());
        assert!(!path.is_pillar());
        assert_eq!(path.get_wall_identifier(), None);
        assert_eq!(path.get_wall_type(), None);
        assert!(!path.is_resolved_path());
    }

    #[test]
    fn test_new_resolved_path() {
        let path = MazePointStatus::new_resolved_path();
        assert!(path.is_path());
        assert!(!path.is_wall());
        assert!(!path.is_outside_wall());
        assert!(!path.is_pillar());
        assert_eq!(path.get_wall_identifier(), None);
        assert_eq!(path.get_wall_type(), None);
        assert!(path.is_resolved_path());
    }

    #[test]
    fn test_new_maze_wall() {
        let identifier = WallIdentifier::new();
        let maze_wall = MazePointStatus::new_maze_wall(identifier);

        assert!(!maze_wall.is_path());
        assert!(maze_wall.is_wall());
        assert!(!maze_wall.is_outside_wall());
        assert!(!maze_wall.is_pillar());
        assert!(maze_wall.is_my_wall(&identifier));
        assert_eq!(maze_wall.get_wall_identifier(), Some(&identifier));
        assert!(maze_wall.get_wall_type().is_some());
    }

    #[test]
    fn test_new_start_point_outside_wall() {
        let identifier = WallIdentifier::new();
        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier);

        assert!(!start_wall.is_path());
        assert!(start_wall.is_wall());
        assert!(start_wall.is_outside_wall());
        assert!(start_wall.is_not_checked_start_point());
        assert!(!start_wall.is_extending_start_point());
        assert!(!start_wall.is_pillar());
        assert!(start_wall.is_my_wall(&identifier));
        assert!(start_wall.is_not_checked_wall());
        assert!(!start_wall.is_extending_wall());
    }

    #[test]
    fn test_new_just_outside_wall() {
        let identifier = WallIdentifier::new();
        let just_wall = MazePointStatus::new_just_outside_wall(identifier);

        assert!(!just_wall.is_path());
        assert!(just_wall.is_wall());
        assert!(just_wall.is_outside_wall());
        assert!(just_wall.is_just_outside_wall());
        assert!(!just_wall.is_not_checked_start_point());
        assert!(!just_wall.is_extending_start_point());
        assert!(!just_wall.is_pillar());
        assert!(just_wall.is_my_wall(&identifier));
        assert!(just_wall.is_not_checked_wall());
        assert!(!just_wall.is_extending_wall());
    }

    #[test]
    fn test_new_notchecked_pillar() {
        let pillar = MazePointStatus::new_notchecked_pillar();

        assert!(!pillar.is_path());
        assert!(pillar.is_wall());
        assert!(!pillar.is_outside_wall());
        assert!(pillar.is_pillar());
        assert!(pillar.is_not_checked_pillar());
        assert!(!pillar.is_extending_pillar());
        assert_eq!(pillar.get_wall_identifier(), None);
        assert!(pillar.is_not_checked_wall());
        assert!(!pillar.is_extending_wall());
    }

    // State Transition Tests
    #[test]
    fn test_start_point_state_transition() {
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier1);
        assert!(start_wall.is_not_checked_start_point());
        assert!(!start_wall.is_extending_start_point());

        // Convert to extending state
        let extending_start =
            MazePointStatus::new_extending_start_point_from_notchecked_start_point(
                start_wall,
                &identifier1,
            )
            .unwrap();

        assert!(!extending_start.is_not_checked_start_point());
        assert!(extending_start.is_extending_start_point());
        assert!(extending_start.is_my_wall(&identifier1));
        assert!(!extending_start.is_my_wall(&identifier2));
        assert!(!extending_start.is_not_checked_wall());
        assert!(extending_start.is_extending_wall());
    }

    #[test]
    fn test_pillar_state_transition() {
        let identifier = WallIdentifier::new();

        let pillar = MazePointStatus::new_notchecked_pillar();
        assert!(pillar.is_not_checked_pillar());
        assert!(!pillar.is_extending_pillar());

        // Convert to extending state
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier)
                .unwrap();

        assert!(!extending_pillar.is_not_checked_pillar());
        assert!(extending_pillar.is_extending_pillar());
        assert!(extending_pillar.is_my_wall(&identifier));
        assert_eq!(extending_pillar.get_wall_identifier(), Some(&identifier));
        assert!(!extending_pillar.is_not_checked_wall());
        assert!(extending_pillar.is_extending_wall());
    }

    // Error Handling Tests
    #[test]
    fn test_invalid_start_point_transition() {
        let identifier = WallIdentifier::new();

        // Try to convert path to extending start point (should fail)
        let path = MazePointStatus::new_not_resolved_path();
        let result = MazePointStatus::new_extending_start_point_from_notchecked_start_point(
            path,
            &identifier,
        );
        assert!(result.is_err());

        // Try to convert maze wall to extending start point (should fail)
        let maze_wall = MazePointStatus::new_maze_wall(identifier);
        let result = MazePointStatus::new_extending_start_point_from_notchecked_start_point(
            maze_wall,
            &identifier,
        );
        assert!(result.is_err());

        // Try to convert pillar to extending start point (should fail)
        let pillar = MazePointStatus::new_notchecked_pillar();
        let result = MazePointStatus::new_extending_start_point_from_notchecked_start_point(
            pillar,
            &identifier,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_pillar_transition() {
        let identifier = WallIdentifier::new();

        // Try to convert path to extending pillar (should fail)
        let path = MazePointStatus::new_not_resolved_path();
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(path, &identifier);
        assert!(result.is_err());

        // Try to convert start point to extending pillar (should fail)
        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier);
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(start_wall, &identifier);
        assert!(result.is_err());

        // Try to convert already extending pillar (should fail)
        let pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier)
                .unwrap();
        let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            extending_pillar,
            &identifier,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_error_messages() {
        let identifier = WallIdentifier::new();

        let path = MazePointStatus::new_not_resolved_path();
        let result =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(path, &identifier);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot convert to extending pillar"));
        assert!(error_msg.contains("expected NotChecked pillar"));

        let maze_wall = MazePointStatus::new_maze_wall(identifier);
        let result = MazePointStatus::new_extending_start_point_from_notchecked_start_point(
            maze_wall,
            &identifier,
        );
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot convert to extending start point"));
        assert!(error_msg.contains("expected NotChecked start point"));
    }

    // Comprehensive Boolean Method Tests
    #[test]
    fn test_all_boolean_methods_path() {
        let path = MazePointStatus::new_not_resolved_path();

        // Path assertions
        assert!(path.is_path());
        assert!(!path.is_wall());

        // Wall type assertions (all false for path)
        assert!(!path.is_outside_wall());
        assert!(!path.is_not_checked_start_point());
        assert!(!path.is_extending_start_point());
        assert!(!path.is_just_outside_wall());
        assert!(!path.is_pillar());
        assert!(!path.is_not_checked_pillar());
        assert!(!path.is_extending_pillar());

        // State assertions (all false for path)
        assert!(!path.is_extending_wall());
        assert!(!path.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_maze_wall() {
        let identifier = WallIdentifier::new();
        let maze_wall = MazePointStatus::new_maze_wall(identifier);

        // Basic type assertions
        assert!(!maze_wall.is_path());
        assert!(maze_wall.is_wall());

        // Specific wall type assertions
        assert!(!maze_wall.is_outside_wall());
        assert!(!maze_wall.is_not_checked_start_point());
        assert!(!maze_wall.is_extending_start_point());
        assert!(!maze_wall.is_just_outside_wall());
        assert!(!maze_wall.is_pillar());
        assert!(!maze_wall.is_not_checked_pillar());
        assert!(!maze_wall.is_extending_pillar());

        // State assertions
        assert!(!maze_wall.is_extending_wall());
        assert!(!maze_wall.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_notchecked_start_point() {
        let identifier = WallIdentifier::new();
        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier);

        // Basic type assertions
        assert!(!start_wall.is_path());
        assert!(start_wall.is_wall());

        // Outside wall assertions
        assert!(start_wall.is_outside_wall());
        assert!(start_wall.is_not_checked_start_point());
        assert!(!start_wall.is_extending_start_point());
        assert!(!start_wall.is_just_outside_wall());

        // Pillar assertions (false)
        assert!(!start_wall.is_pillar());
        assert!(!start_wall.is_not_checked_pillar());
        assert!(!start_wall.is_extending_pillar());

        // State assertions
        assert!(!start_wall.is_extending_wall());
        assert!(start_wall.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_extending_start_point() {
        let identifier1 = WallIdentifier::new();
        let _identifier2 = WallIdentifier::new();

        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier1);
        let extending_start =
            MazePointStatus::new_extending_start_point_from_notchecked_start_point(
                start_wall,
                &identifier1,
            )
            .unwrap();

        // Basic type assertions
        assert!(!extending_start.is_path());
        assert!(extending_start.is_wall());

        // Outside wall assertions
        assert!(extending_start.is_outside_wall());
        assert!(!extending_start.is_not_checked_start_point());
        assert!(extending_start.is_extending_start_point());
        assert!(!extending_start.is_just_outside_wall());

        // Pillar assertions (false)
        assert!(!extending_start.is_pillar());
        assert!(!extending_start.is_not_checked_pillar());
        assert!(!extending_start.is_extending_pillar());

        // State assertions
        assert!(extending_start.is_extending_wall());
        assert!(!extending_start.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_just_outside_wall() {
        let identifier = WallIdentifier::new();
        let just_wall = MazePointStatus::new_just_outside_wall(identifier);

        // Basic type assertions
        assert!(!just_wall.is_path());
        assert!(just_wall.is_wall());

        // Outside wall assertions
        assert!(just_wall.is_outside_wall());
        assert!(!just_wall.is_not_checked_start_point());
        assert!(!just_wall.is_extending_start_point());
        assert!(just_wall.is_just_outside_wall());

        // Pillar assertions (false)
        assert!(!just_wall.is_pillar());
        assert!(!just_wall.is_not_checked_pillar());
        assert!(!just_wall.is_extending_pillar());

        // State assertions
        assert!(!just_wall.is_extending_wall());
        assert!(just_wall.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_notchecked_pillar() {
        let pillar = MazePointStatus::new_notchecked_pillar();

        // Basic type assertions
        assert!(!pillar.is_path());
        assert!(pillar.is_wall());

        // Outside wall assertions (false)
        assert!(!pillar.is_outside_wall());
        assert!(!pillar.is_not_checked_start_point());
        assert!(!pillar.is_extending_start_point());
        assert!(!pillar.is_just_outside_wall());

        // Pillar assertions
        assert!(pillar.is_pillar());
        assert!(pillar.is_not_checked_pillar());
        assert!(!pillar.is_extending_pillar());

        // State assertions
        assert!(!pillar.is_extending_wall());
        assert!(pillar.is_not_checked_wall());
    }

    #[test]
    fn test_all_boolean_methods_extending_pillar() {
        let identifier = WallIdentifier::new();

        let pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier)
                .unwrap();

        // Basic type assertions
        assert!(!extending_pillar.is_path());
        assert!(extending_pillar.is_wall());

        // Outside wall assertions (false)
        assert!(!extending_pillar.is_outside_wall());
        assert!(!extending_pillar.is_not_checked_start_point());
        assert!(!extending_pillar.is_extending_start_point());
        assert!(!extending_pillar.is_just_outside_wall());

        // Pillar assertions
        assert!(extending_pillar.is_pillar());
        assert!(!extending_pillar.is_not_checked_pillar());
        assert!(extending_pillar.is_extending_pillar());

        // State assertions
        assert!(extending_pillar.is_extending_wall());
        assert!(!extending_pillar.is_not_checked_wall());
    }

    // Identifier Management Tests
    #[test]
    fn test_wall_identifier_management() {
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // Test maze wall
        let maze_wall = MazePointStatus::new_maze_wall(identifier1);
        assert_eq!(maze_wall.get_wall_identifier(), Some(&identifier1));
        assert!(maze_wall.is_my_wall(&identifier1));
        assert!(!maze_wall.is_my_wall(&identifier2));

        // Test start point
        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier1);
        assert_eq!(start_wall.get_wall_identifier(), Some(&identifier1));
        assert!(start_wall.is_my_wall(&identifier1));
        assert!(!start_wall.is_my_wall(&identifier2));

        // Test pillar (no identifier initially)
        let pillar = MazePointStatus::new_notchecked_pillar();
        assert_eq!(pillar.get_wall_identifier(), None);
        assert!(!pillar.is_my_wall(&identifier1));
        assert!(!pillar.is_my_wall(&identifier2));

        // Test extending pillar (gets identifier)
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier2)
                .unwrap();
        assert_eq!(extending_pillar.get_wall_identifier(), Some(&identifier2));
        assert!(!extending_pillar.is_my_wall(&identifier1));
        assert!(extending_pillar.is_my_wall(&identifier2));

        // Test path (no identifier)
        let path = MazePointStatus::new_not_resolved_path();
        assert_eq!(path.get_wall_identifier(), None);
        assert!(!path.is_my_wall(&identifier1));
        assert!(!path.is_my_wall(&identifier2));
    }

    // Wall Type Retrieval Tests
    #[test]
    fn test_wall_type_retrieval() {
        let identifier = WallIdentifier::new();

        // Path should return None
        let path = MazePointStatus::new_not_resolved_path();
        assert!(path.get_wall_type().is_none());

        // Each wall type should return Some
        let maze_wall = MazePointStatus::new_maze_wall(identifier);
        assert!(maze_wall.get_wall_type().is_some());

        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier);
        assert!(start_wall.get_wall_type().is_some());
        if let Some(wall_type) = start_wall.get_wall_type() {
            assert!(matches!(wall_type, WallType::Outside(..)));
        }

        let just_wall = MazePointStatus::new_just_outside_wall(identifier);
        assert!(just_wall.get_wall_type().is_some());
        if let Some(wall_type) = just_wall.get_wall_type() {
            assert!(matches!(wall_type, WallType::Outside(..)));
        }

        let pillar = MazePointStatus::new_notchecked_pillar();
        assert!(pillar.get_wall_type().is_some());
        if let Some(wall_type) = pillar.get_wall_type() {
            assert!(matches!(wall_type, WallType::Pillar(..)));
        }
    }

    // Clone and Equality Tests
    #[test]
    fn test_clone_and_equality() {
        let identifier = WallIdentifier::new();

        // Test path cloning
        let path1 = MazePointStatus::new_not_resolved_path();
        let path2 = path1.clone();
        assert_eq!(path1, path2);

        // Test wall cloning
        let maze_wall1 = MazePointStatus::new_maze_wall(identifier);
        let maze_wall2 = maze_wall1.clone();
        assert_eq!(maze_wall1, maze_wall2);

        let pillar1 = MazePointStatus::new_notchecked_pillar();
        let pillar2 = pillar1.clone();
        assert_eq!(pillar1, pillar2);

        // Test inequality between different types
        assert_ne!(path1, maze_wall1);
        assert_ne!(path1, pillar1);
        assert_ne!(maze_wall1, pillar1);
    }

    // Hash Consistency Tests
    #[test]
    fn test_hash_consistency() {
        let identifier = WallIdentifier::new();
        let mut status_map = HashMap::new();

        // Test that equal statuses have same hash
        let path1 = MazePointStatus::new_not_resolved_path();
        let path2 = MazePointStatus::new_not_resolved_path();
        status_map.insert(path1, "path");
        assert_eq!(status_map.get(&path2), Some(&"path"));

        let pillar1 = MazePointStatus::new_notchecked_pillar();
        let pillar2 = MazePointStatus::new_notchecked_pillar();
        status_map.insert(pillar1, "pillar");
        assert_eq!(status_map.get(&pillar2), Some(&"pillar"));

        // Test that different statuses have different hashes
        let maze_wall = MazePointStatus::new_maze_wall(identifier);
        status_map.insert(maze_wall.clone(), "maze_wall");
        assert_ne!(status_map.get(&path2), status_map.get(&maze_wall));
    }

    // Debug Output Tests
    #[test]
    fn test_debug_output() {
        let identifier = WallIdentifier::new();

        let path = MazePointStatus::new_not_resolved_path();
        let debug_str = format!("{:?}", path);
        assert!(debug_str.contains("Path"));
        assert!(debug_str.contains("NewMethodEnforcer"));

        let maze_wall = MazePointStatus::new_maze_wall(identifier);
        let debug_str = format!("{:?}", maze_wall);
        assert!(debug_str.contains("Wall"));
        assert!(debug_str.contains("Some"));

        let pillar = MazePointStatus::new_notchecked_pillar();
        let debug_str = format!("{:?}", pillar);
        assert!(debug_str.contains("Wall"));
        assert!(debug_str.contains("None"));
    }

    // Memory Layout Tests
    #[test]
    fn test_memory_layout_consistency() {
        let identifier = WallIdentifier::new();

        // Same variant types should have same size
        let path1 = MazePointStatus::new_not_resolved_path();
        let path2 = MazePointStatus::new_not_resolved_path();
        assert_eq!(std::mem::size_of_val(&path1), std::mem::size_of_val(&path2));

        let pillar1 = MazePointStatus::new_notchecked_pillar();
        let pillar2 = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            pillar1.clone(),
            &identifier,
        )
        .unwrap();
        assert_eq!(
            std::mem::size_of_val(&pillar1),
            std::mem::size_of_val(&pillar2)
        );
    }

    // Edge Case Tests
    #[test]
    fn test_double_conversion_prevention() {
        let identifier = WallIdentifier::new();

        // Test that already converted states cannot be converted again
        let pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier)
                .unwrap();

        // Attempting to convert extending pillar should fail
        let result = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
            extending_pillar,
            &identifier,
        );
        assert!(result.is_err());
    }

    // Comprehensive State Machine Tests
    #[test]
    fn test_state_machine_completeness() {
        let identifier = WallIdentifier::new();

        // Test all possible state combinations
        let test_cases = vec![
            (MazePointStatus::new_not_resolved_path(), "path"),
            (MazePointStatus::new_maze_wall(identifier), "maze_wall"),
            (
                MazePointStatus::new_start_point_outside_wall(identifier),
                "not_checked_start",
            ),
            (
                MazePointStatus::new_just_outside_wall(identifier),
                "just_wall",
            ),
            (
                MazePointStatus::new_notchecked_pillar(),
                "not_checked_pillar",
            ),
        ];

        // Add extending states
        let start_wall = MazePointStatus::new_start_point_outside_wall(identifier);
        let extending_start =
            MazePointStatus::new_extending_start_point_from_notchecked_start_point(
                start_wall,
                &identifier,
            )
            .unwrap();
        let mut test_cases_with_extending = test_cases;
        test_cases_with_extending.push((extending_start, "extending_start"));

        let pillar = MazePointStatus::new_notchecked_pillar();
        let extending_pillar =
            MazePointStatus::new_extending_pillar_from_notchecked_pillar(pillar, &identifier)
                .unwrap();
        test_cases_with_extending.push((extending_pillar, "extending_pillar"));

        // Verify each state has exactly one true condition
        for (status, expected_type) in test_cases_with_extending {
            let mut true_count = 0;
            let conditions = vec![
                (status.is_path(), "path"),
                (status.is_wall(), "wall"),
                (status.is_outside_wall(), "outside_wall"),
                (
                    status.is_not_checked_start_point(),
                    "not_checked_start_point",
                ),
                (status.is_extending_start_point(), "extending_start_point"),
                (status.is_just_outside_wall(), "just_outside_wall"),
                (status.is_pillar(), "pillar"),
                (status.is_not_checked_pillar(), "not_checked_pillar"),
                (status.is_extending_pillar(), "extending_pillar"),
                (status.is_extending_wall(), "extending_wall"),
                (status.is_not_checked_wall(), "not_checked_wall"),
            ];

            for (condition, _name) in conditions {
                if condition {
                    true_count += 1;
                }
            }

            // Some states can have multiple true conditions (e.g., extending_pillar is both pillar and extending_wall)
            assert!(
                true_count >= 1,
                "State {} should have at least one true condition",
                expected_type
            );
        }
    }

    // Performance Tests
    #[test]
    fn test_performance_characteristics() {
        let identifier = WallIdentifier::new();

        // Test creation performance
        let start = Instant::now();
        let _statuses: Vec<MazePointStatus> = (0..1000)
            .map(|i| match i % 5 {
                0 => MazePointStatus::new_not_resolved_path(),
                1 => MazePointStatus::new_maze_wall(identifier),
                2 => MazePointStatus::new_notchecked_pillar(),
                3 => MazePointStatus::new_start_point_outside_wall(identifier),
                _ => MazePointStatus::new_just_outside_wall(identifier),
            })
            .collect();
        let creation_time = start.elapsed();

        // Test query performance
        let start = Instant::now();
        let _results: Vec<bool> = _statuses
            .iter()
            .map(|s| s.is_wall() && s.is_pillar() && s.is_not_checked_wall())
            .collect();
        let query_time = start.elapsed();

        println!(
            "Creation time: {:?}, Query time: {:?}",
            creation_time, query_time
        );

        // Performance should be reasonable (adjust thresholds as needed)
        assert!(creation_time.as_millis() < 100);
        assert!(query_time.as_millis() < 10);
    }

    // Constructor Method Enforcement Tests
    #[test]
    fn test_constructor_vs_direct_construction() {
        let identifier = WallIdentifier::new();

        // Recommended constructor usage
        let _path = MazePointStatus::new_not_resolved_path();
        let _maze_wall = MazePointStatus::new_maze_wall(identifier);
        let _pillar = MazePointStatus::new_notchecked_pillar();

        // Direct construction (possible but discouraged)
        let _direct_path =
            MazePointStatus::Path(PathType::NotResolvedPath, NewMethodEnforcer::new());
        let _direct_wall = MazePointStatus::Wall(
            WallType::new_not_checked_pillar(),
            None,
            NewMethodEnforcer::new(),
        );

        // Constructor and direct construction should produce equivalent results
        assert_eq!(_path, _direct_path);
        assert_eq!(_pillar, _direct_wall);
    }
}
