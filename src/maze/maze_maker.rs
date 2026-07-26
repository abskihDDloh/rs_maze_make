use std::
    sync::{Arc, RwLock}
;

use log::{debug, warn};
use rand::RngExt;

use crate::maze::{
    maze_cell::{maze_point::point::MazePoint, wall::wall_identifier::WallIdentifier},
    maze_field::{extend_result::ExtendResult, field::Field},
};

/// 利用可能な外壁開始点（未探索のStartPoint）をランダムに選び、Extending状態に変更して返します。
///
/// # 概要
/// - 迷路外周の未探索開始点からランダムに1つを選択し、状態をExtendingに変更します。
/// - スレッドセーフで、複数スレッドから同時に呼び出しても安全です。
/// - 候補がなければエラーを返します。
///
/// # 引数
/// * `maze_points` - 迷路フィールド（共有参照）
/// * `identifier` - 割り当てる壁の識別子
///
/// # 戻り値
/// * `Ok(MazePoint)` - 選択された開始点（Extending状態に変更済み）
/// * `Err(_)` - 利用可能な開始点がない、またはロック取得失敗等
///
/// # エラー条件
/// - 全開始点が探索済み
/// - 利用可能な開始点が見つからない
/// - ロック取得失敗
/// - 状態変換失敗
///
/// # 使用例
/// ```rust
/// let field = Arc::new(RwLock::new(Field::new(7, 7)?));
/// let identifier = WallIdentifier::new();
/// let start = select_start_point_outside_wall(&field, identifier)?;
/// println!("Start: {:?}", start);
/// ```
///
/// # 関連
/// - [`Field::get_random_available_start_point()`]
/// - [`Field::mark_start_point_as_extending()`]
/// - [`WallIdentifier`]
/// - [`OutsideWallType::StartPoint`]
#[allow(dead_code)]
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

pub(in crate::maze) fn select_start_point_any_source(
    maze_points: &Arc<RwLock<Field>>,
    partition_id: usize,
    identifier: WallIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(" identifier: {}", identifier.as_str());

    let mut maze_points_write = maze_points.write().map_err(|_| {
        Box::new(std::io::Error::other(format!(
            "Failed to acquire write lock. {}",
            error_msg_common_part
        ))) as Box<dyn std::error::Error + Send + Sync>
    })?;

    maze_points_write
        .select_start_source_for_partition(partition_id, &identifier)
        .map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "No available start point or extending pillar source found.{} partition:{} err:{}",
                    error_msg_common_part, partition_id, e
                ),
            )) as Box<dyn std::error::Error + Send + Sync>
        })
}

/// 柱または外壁開始点（Extending状態）から隣接する柱への拡張を試みます。
///
/// # 概要
/// - 指定した柱から距離2の隣接柱候補を探索し、ランダムに1つ選んで拡張します。
/// - 成功時は元柱を拡張済みに、隣接柱をExtendingに、中間点をWallに変更します。
/// - 候補がなければ元柱を拡張済みにし、SurroundedPillarを返します。
/// - 境界や異なる識別子の柱に到達した場合はOutsideを返します。
/// - スレッドセーフで、ロック取得や状態変換失敗時はリトライします。
///
/// # 引数
/// * `maze_points` - 迷路データ（共有参照）
/// * `source_point` - 拡張元の柱座標（Extending状態）
/// * `identifier` - 壁の識別子
///
/// # 戻り値
/// * `Ok(ExtendResult)` - 拡張結果（NextPillar/Outside/SurroundedPillar）
/// * `Err(_)` - ロック取得失敗や不正な状態等
///
/// # エラー条件
/// - 指定柱がExtendingでない
/// - 識別子不一致
/// - 中間点がPathでない
/// - ロック取得失敗
/// - 状態変換失敗
///
/// # 使用例
/// ```rust
/// let maze_points = Arc::new(RwLock::new(Field::new(7, 7)?));
/// let identifier = WallIdentifier::new();
/// let start = select_start_point_outside_wall(&maze_points, identifier.clone())?;
/// let result = extend_point_to_adjacent_pillar(&maze_points, start, identifier)?;
/// println!("Result: {:?}", result);
/// ```
///
/// # 関連
/// - [`Field::get_adjacent_extendable_pillars()`]
/// - [`Field::execute_pillar_extension()`]
/// - [`ExtendResult`]
pub(in crate::maze) fn extend_point_to_adjacent_pillar(
    maze_points: &Arc<RwLock<Field>>,
    source_point: MazePoint,
    partition_id: usize,
    identifier: WallIdentifier,
) -> Result<ExtendResult, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(
        " identifier: {} from_pillar {:?}",
        identifier.as_str(),
        source_point
    );
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

    let adjacent_pillars_candidate =
        maze_points_write.get_adjacent_extendable_pillars_for_partition(&source_point, partition_id);

    if adjacent_pillars_candidate.is_empty() {
        debug!(
            "adjacent pillars not found. return. {}",
            error_msg_common_part
        );
        return Ok(ExtendResult::new_extending_pillar());
    }

    let mut rng = rand::rng();
    let selected_pillar =
        adjacent_pillars_candidate[rng.random_range(0..adjacent_pillars_candidate.len())];

    let result = maze_points_write.execute_pillar_extension(&source_point, &selected_pillar, &identifier);

    match result {
        Err(e) => {
            warn!("Error occurred: {}", e);
            Err(e)
        }
        Ok(r) => {
            debug!("extend_status: {:?} {}", r, error_msg_common_part);
            Ok(r)
        }
    }
}
