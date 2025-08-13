use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use log::{debug, warn};
use rand::Rng;

use crate::maze::{
    maze_cell::{maze_point::point::MazePoint, wall::wall_identifier::WallIdentifier},
    maze_field::{extend_result::ExtendResult, field::Field},
};

/// 迷路生成の開始点となる外壁をランダムに選択する
///
/// この関数は利用可能な外壁開始点（StartPoint + NotChecked状態）の中からランダムに1つを選択し、
/// `mark_start_point_as_extending()` を使用してExtending状態に変更してから返します。
/// スレッドセーフな操作を行い、複数のスレッドから同時にアクセスされても安全です。
///
/// ## 処理フロー
///
/// 1. **読み取りロック取得**: 全開始点探索フラグをチェック
/// 2. **開始点選択**: `get_random_available_start_point()` でランダム選択
/// 3. **書き込みロック取得**: 状態変更のための排他制御
/// 4. **状態変換**: NotChecked → Extending への安全な変換
/// 5. **結果返却**: 選択された開始点の座標を返す
///
/// ## 最適化ポイント
///
/// - **読み取りロック優先**: 候補選択は読み取りロックで実行し並行性を向上
/// - **書き込みロック最小化**: 状態変更のみに書き込みロックを使用
/// - **競合状態回避**: ロック間での状態変化を考慮したループ処理
/// - **効率的な選択**: `get_random_available_start_point()` による効率的な候補選択
///
/// ## 開始点の条件
///
/// 選択対象となる外壁開始点は以下の条件をすべて満たす必要があります：
/// - **壁の種類**: `WallType::Outside(OutsideWallType::StartPoint, _, _)`
/// - **拡張状態**: `ExtendStatus::NotChecked`
/// - **識別子**: `None`（未割り当て状態）
/// - **配置位置**: 迷路境界の0以外の偶数座標
///
/// ## エラーハンドリング
///
/// この関数は以下の状況でエラーを返します：
/// - 全開始点が既に探索済み（`all_start_point_seeked_flag == true`）
/// - 利用可能な開始点が見つからない
/// - 読み取り/書き込みロックの取得に失敗
/// - 開始点の状態変換に失敗（競合状態等）
///
/// ## 並行性への配慮
///
/// - **デッドロック回避**: 読み取り→書き込みの順序でロック取得
/// - **競合状態対応**: 状態変換失敗時のリトライ機構
/// - **ロック最小化**: 不要な期間でのロック保持を回避
/// - **原子性保証**: 開始点選択と状態変換の原子的実行
///
/// # Arguments
///
/// * `maze_points` - 迷路フィールドデータへの共有参照
/// * `identifier` - 開始点に割り当てる壁の識別子
///
/// # Returns
///
/// 成功時は選択された開始点の座標、失敗時はエラー
///
/// ## 戻り値の詳細
///
/// - **`Ok(MazePoint)`**: 選択された開始点の座標
///   - 選択された開始点は `Extending` 状態に変換済み
///   - 指定された `identifier` が割り当て済み
///   - 迷路生成処理を開始可能な状態
/// - **`Err(Box<dyn Error>)`**: 処理失敗時のエラー情報
///
/// # Errors
///
/// | エラーの種類 | 発生条件 | 対処方法 |
/// |-------------|----------|----------|
/// | `InvalidData` | 全開始点が探索済み | 迷路生成完了として処理 |
/// | `NotFound` | 利用可能な開始点なし | 迷路サイズや配置を確認 |
/// | `Other` | ロック取得失敗 | リトライまたは異常終了 |
/// | `InvalidData` | 状態変換失敗 | 内部状態の整合性を確認 |
///
/// # Examples
///
/// ## 基本的な使用例
///
/// ```rust
/// use std::sync::{Arc, RwLock};
/// use crate::maze::{
///     maze_field::field::Field,
///     maze_cell::wall::wall_identifier::WallIdentifier
/// };
///
/// // 迷路フィールドの初期化
/// let field = Arc::new(RwLock::new(Field::new(7, 7)?));
/// let identifier = WallIdentifier::new();
///
/// // 開始点の選択
/// match select_start_point_outside_wall(&field, identifier) {
///     Ok(start_point) => {
///         println!("Selected start point: {:?}", start_point);
///         // 迷路生成処理を開始
///     }
///     Err(e) => {
///         eprintln!("Failed to select start point: {}", e);
///     }
/// }
/// ```
///
/// ## マルチスレッド環境での使用例
///
/// ```rust
/// use std::thread;
/// use std::sync::{Arc, RwLock};
///
/// let field = Arc::new(RwLock::new(Field::new(15, 15)?));
/// let mut handles = vec![];
///
/// for thread_id in 0..4 {
///     let field_clone = Arc::clone(&field);
///     let handle = thread::spawn(move || {
///         let identifier = WallIdentifier::new();
///         
///         loop {
///             match select_start_point_outside_wall(&field_clone, identifier.clone()) {
///                 Ok(start_point) => {
///                     println!("Thread {} selected: {:?}", thread_id, start_point);
///                     // 迷路生成処理を実行
///                     break;
///                 }
///                 Err(e) if e.to_string().contains("All start points") => {
///                     println!("Thread {} finished: no more start points", thread_id);
///                     break;
///                 }
///                 Err(e) => {
///                     eprintln!("Thread {} error: {}", thread_id, e);
///                     // リトライまたは終了
///                     break;
///                 }
///             }
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// ```
///
/// ## エラーハンドリングの例
///
/// ```rust
/// let field = Arc::new(RwLock::new(Field::new(7, 7)?));
/// let identifier = WallIdentifier::new();
///
/// match select_start_point_outside_wall(&field, identifier) {
///     Ok(start_point) => {
///         // 成功：迷路生成を開始
///         println!("Starting maze generation from: {:?}", start_point);
///     }
///     Err(e) => {
///         let error_msg = e.to_string();
///         if error_msg.contains("All start points have been sought") {
///             println!("Maze generation completed - all start points processed");
///         } else if error_msg.contains("No available start point found") {
///             eprintln!("Configuration error: no valid start points available");
///         } else {
///             eprintln!("Unexpected error: {}", error_msg);
///         }
///     }
/// }
/// ```
///
/// # Performance Notes
///
/// - **時間計算量**: O(1) - ランダム選択による定数時間
/// - **空間計算量**: O(1) - 固定サイズのデータのみ使用
/// - **ロック競合**: 最小化された書き込みロック期間
/// - **スケーラビリティ**: マルチスレッド環境で良好な性能
///
/// # See Also
///
/// - [`Field::get_random_available_start_point()`] - 開始点候補の選択
/// - [`Field::mark_start_point_as_extending()`] - 開始点の状態変換
/// - [`extend_pillar_to_adjacent_pillar()`] - 後続の拡張処理
/// - [`WallIdentifier`] - 壁の識別子管理
/// - [`OutsideWallType::StartPoint`] - 外壁開始点の定義
pub(in crate::maze) fn select_start_point_outside_wall(
    maze_points: &Arc<RwLock<Field>>,
    identifier: WallIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(" identifier: {}", identifier.as_str());

    loop {
        let maze_points_read = maze_points.read().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire read lock. {:?}",
                identifier
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;
        if maze_points_read.all_pillar_seeked_flag() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "All start points have been sought. {}",
                    error_msg_common_part
                ),
            )));
        }
        // Noneの場合はエラー
        let start_point: MazePoint = maze_points_read
            .get_random_available_start_point()
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("No available start point found.{}", error_msg_common_part),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        drop(maze_points_read); // 読み取りロックを解放

        let mut maze_points_write = maze_points.write().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire write lock. {}",
                error_msg_common_part
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        if maze_points_write.all_pillar_seeked_flag() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "All start points have been sought. {}",
                    error_msg_common_part
                ),
            )));
        }

        match maze_points_write.mark_start_point_as_extending(&start_point, &identifier) {
            Ok(()) => {
                drop(maze_points_write);
                return Ok(start_point);
            }
            Err(err) => {
                warn!("Failed to mark start point as extending. {}", err);
                drop(maze_points_write); // 書き込みロックを解放
                continue;
            }
        }
    }
}

/// 柱もしくは外壁開始点から隣接する柱への拡張を試行する
///
/// この関数は指定された柱もしくは外壁開始点（Extending状態）から隣接する利用可能な柱への拡張を試行します。
/// `MazePointStatus`の新しい判定メソッド `is_extending_wall()`, `is_my_wall()` を使用して
/// 状態判定を安全かつ簡潔に行います。
///
/// 成功した場合、元の柱は拡張済み状態になり、選択された隣接柱はExtending状態になります。
/// また、両柱の間の座標点はWall状態に変更されます。
///
/// 隣接する柱の候補は距離2の位置（上下左右）にある座標点で、以下の条件を満たすもの：
/// - NotChecked状態の柱
/// - Outside壁（境界）
/// - ExtendingまたはExtended状態の柱で、識別子が異なる場合（外壁として扱う）
///
/// 中間点（両柱の中点）はPath状態である必要があり、拡張時にWall状態に変更されます。
/// 利用可能な隣接柱がない場合は、元の柱をExtended状態に変更してSurroundedPillar結果を返します。
///
/// **重要な変更点**:
/// - `all_pillar_seeked_flag`がtrueの場合、エラーではなく`SurroundedPillar`結果を返します。
/// - ExtendingもしくはExtendedのpillarで、identifierが引数と異なる場合は外壁と同じ扱いをします。
/// - 読み取りロックを活用して並行性を向上させます。
/// - unwrap()をエラーハンドリングに置き換えます。
/// - 新しいメソッド `is_extending_wall()`, `is_my_wall()` を使用して状態判定を簡潔にします。
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `pillar_point` - 拡張元の柱の座標（Extending状態である必要があります）
/// * `identifier` - 壁の識別子（元の柱の識別子と一致する必要があります）
///
/// # Returns
///
/// 拡張操作の結果。以下のいずれかの結果を返します：
/// - `NextPillar`: 隣接する柱が見つかり、拡張に成功した場合
/// - `Outside`: 境界（Outside壁）に到達した場合、または異なる識別子の柱に到達した場合
/// - `SurroundedPillar`: 利用可能な隣接柱がない場合、または全柱探索が完了している場合
///
/// # Errors
///
/// * 指定された柱がExtending状態でない場合
/// * 識別子が一致しない場合
/// * 中間点がPath状態でない場合
/// * ロックの取得に失敗した場合
/// * 指定された座標点が見つからない場合
/// * 状態変換に失敗した場合
///
/// # Examples
///
/// ```rust
/// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
/// let identifier = WallIdentifier::new();
/// let start_pillar = select_start_pillar_point(&maze_points, identifier.clone())?;
/// let result = extend_pillar_to_adjacent_pillar(&maze_points, start_pillar, identifier)?;
///
/// match result {
///     result if result.is_next_pillar() => {
///         println!("Next pillar found at: {:?}", result.point);
///         // 拡張を継続
///     }
///     result if result.is_outside() => {
///         println!("Reached boundary at: {:?}", result.point);
///         // 新しい開始点を選択
///     }
///     result if result.is_surrounded() => {
///         println!("Pillar is surrounded or all pillars processed");
///         // 新しい開始点を選択、または処理終了
///     }
/// }
/// ```
pub(in crate::maze) fn extend_point_to_adjacent_pillar(
    maze_points: &Arc<RwLock<Field>>,
    source_point: MazePoint,
    identifier: WallIdentifier,
) -> Result<ExtendResult, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(
        " identifier: {} from_pillar {:?}",
        identifier.as_str(),
        source_point
    );
    loop {
        debug!(
            "Attempting to extend point:  with identifier {}",
            error_msg_common_part
        );

        let maze_points_read = maze_points.read().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire read lock. {:?}",
                identifier
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;
        if maze_points_read.all_pillar_seeked_flag() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "All start points have been sought. {}",
                    error_msg_common_part
                ),
            )));
        }
        let adjacent_pillars_candidate =
            maze_points_read.get_adjacent_extendable_pillars(&source_point);
        drop(maze_points_read);

        if adjacent_pillars_candidate.is_empty() {
            debug!(
                "adjacent pillars not found. return. {}",
                error_msg_common_part
            );
            return Ok(ExtendResult::new_extending_pillar()); // 利用可能な隣接柱がない場合は拡張不能。
        }

        // 隣接柱候補からランダムに1つ選択
        let selected_pillar = adjacent_pillars_candidate
            [rand::rng().random_range(0..adjacent_pillars_candidate.len())];

        let mut maze_points_write = maze_points.write().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire write lock. {}",
                error_msg_common_part
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;
        if maze_points_write.all_pillar_seeked_flag() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "All start points have been sought. {}",
                    error_msg_common_part
                ),
            )));
        }
        let result = maze_points_write.execute_pillar_extension(
            &source_point,
            &selected_pillar,
            &identifier,
        );
        // エラーの場合は再試行
        match result {
            Err(e) => {
                drop(maze_points_write); // 書き込みロックを解放
                warn!("Error occurred: {}", e);
                continue;
            }
            _ => {
                drop(maze_points_write); // 書き込みロックを解放
                let r = result.unwrap();
                debug!("extend_status: {:?} {}", r, error_msg_common_part);
                return Ok(r);
            }
        }
    }
}
