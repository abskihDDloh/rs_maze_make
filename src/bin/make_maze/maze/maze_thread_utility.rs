//! 迷路生成におけるスレッド管理のためのユーティリティ関数群
//!
//! このモジュールは、迷路生成プロセスで使用される各種データベースクエリや
//! スレッド情報の取得・更新を行うヘルパー関数を提供します。
//! 主に以下の機能を含みます：
//!
//! - スレッドレコードの取得
//! - 未使用の柱（開始点）の検索
//! - セルのステータス確認
//! - 外壁からのスレッドの判定
//! - 柱の取得と所有権の更新

use log::debug;
use rand::Rng;
use rs_maze_maker::common::database::entities::temp_outside_wall_start_points;
use rs_maze_maker::common::database::entities::temp_unused_start_points;
use rs_maze_maker::common::database::entities::temp_unused_start_points_count;
use rs_maze_maker::common::database::initializer::populate_temp_outside_wall_start_points;
use rs_maze_maker::common::database::initializer::populate_temp_unused_start_points;
use rs_maze_maker::common::util::get_late_10_percent_flag;
use sea_orm::QuerySelect;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    TransactionTrait,
};

use rs_maze_maker::common::database::entities::maze_cell_owner_view;
use rs_maze_maker::common::database::entities::maze_cell_status_view;
use rs_maze_maker::common::database::entities::maze_field;
use rs_maze_maker::common::database::entities::thread_list;
use rs_maze_maker::common::database::entities::unused_start_points_view;
use rs_maze_maker::common::database::initializer::OutsideWallConnectTypeEnum;
use rs_maze_maker::common::maze_point::MazePoint;

use crate::maze::maze_thread_identifier::MazeThreadIdentifier;

/// 指定されたスレッド識別子に対応するスレッドレコードをデータベースから取得します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `my_thread_id` - 迷路スレッドID文字列
/// * `my_create_unixtime` - 迷路スレッド生成UNIXタイムスタンプ
///
/// # 戻り値
/// スレッドレコードのモデル、またはエラー
///
/// # エラー
/// スレッドレコードが見つからない場合、またはデータベースエラーが発生した場合
pub async fn select_my_thread_record_from_tx(
    txn: &DatabaseTransaction,
    my_thread_id: String,
    my_create_unixtime: i64,
) -> Result<thread_list::Model, Box<dyn std::error::Error>> {
    let my_thread_id = my_thread_id.clone();
    let thread_record = thread_list::Entity::find()
        .filter(
            thread_list::Column::ThreadId
                .eq(my_thread_id.clone())
                .and(thread_list::Column::CreateUnixtime.eq(my_create_unixtime)),
        )
        .one(txn)
        .await?
        .ok_or(format!(
            "Thread record not found. my_thread_id:{:?}, create_unixtime:{:?}",
            my_thread_id, my_create_unixtime
        ))?;
    Ok(thread_record)
}

pub async fn get_unused_points_count_from_temp_records(
    txn: &DatabaseTransaction,
) -> Result<u64, Box<dyn std::error::Error>> {
    // TEMP_UNUSED_START_POINTS_COUNTのレコードを全件取得する。
    let temp_count_records = temp_unused_start_points_count::Entity::find()
        .all(txn)
        .await?;
    //１行ではない場合はエラー。
    if temp_count_records.len() != 1 {
        return Err(format!(
            "TEMP_UNUSED_START_POINTS_COUNT should have exactly one record, but found {} records.",
            temp_count_records.len()
        )
        .into());
    }
    let count = temp_count_records[0].unused_start_points;
    Ok(count)
}

async fn generate_random_unused_start_point_line_number_from_temp_records(
    txn: &DatabaseTransaction,
) -> Result<u64, Box<dyn std::error::Error>> {
    let count = get_unused_points_count_from_temp_records(txn).await?;
    if count == 0 {
        return Err("No unused start points available".into());
    }
    // ランダムなオフセットを生成（0からcount-1の範囲）
    let random_offset = rand::rng().random_range(0..count);
    Ok(random_offset)
}

// 効率的な実装例
async fn get_random_unused_start_point_by_line_number_candidate(
    txn: &DatabaseTransaction,
) -> Result<unused_start_points_view::Model, Box<dyn std::error::Error>> {
    let line_number = generate_random_unused_start_point_line_number_from_temp_records(txn).await?;
    debug!("Generated random line number: {}", line_number);
    // 1. MEMORYテーブルからCELL_IDだけを高速取得
    let cell_id: u64 = temp_unused_start_points::Entity::find()
        .limit(1)
        .offset(line_number)
        .one(txn)
        .await?
        .ok_or("No unused start point found")?
        .cell_id;
    debug!("Selected CELL_ID: {}", cell_id);
    // 2. CELL_IDを使って必要な情報を取得（インデックスで高速）
    let unused_start_point = unused_start_points_view::Entity::find()
        .filter(unused_start_points_view::Column::CellId.eq(cell_id))
        .one(txn)
        .await?
        .ok_or("Cell details not found")?;

    Ok(unused_start_point)
}

pub async fn select_random_start_point(
    txn: &DatabaseTransaction,
) -> Result<unused_start_points_view::Model, Box<dyn std::error::Error>> {
    let res: unused_start_points_view::Model;
    loop {
        let txn_inner = txn.begin().await?;
        let unused_start_points = get_unused_points_count_from_temp_records(&txn_inner).await?;
        if unused_start_points == 0 {
            txn_inner.commit().await?;
            return Err("No unused start points available".into());
        }
        let unused_start_point_res =
            get_random_unused_start_point_by_line_number_candidate(&txn_inner).await;
        match unused_start_point_res {
            Ok(point) => {
                res = point;
                txn_inner.commit().await?;
                break;
            }
            Err(e) => {
                debug!(
                    "First attempt to get random unused start point failed, retrying...: {}",
                    e
                );
                // 取得できなかったらキャッシュを再作成してリトライする。
                populate_temp_unused_start_points(&txn_inner).await?;
                txn_inner.commit().await?;
                continue;
            }
        };
    }
    debug!(
        "Successfully selected random unused start point: cell_id={}, x={}, y={}",
        res.cell_id, res.x, res.y
    );
    Ok(res)
}
/// 指定された柱のリストの中から、未使用の柱を検索します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `pillars` - チェックする柱の座標リスト
///
/// # 戻り値
/// 未使用開始点ビューのモデルのベクタ、またはエラー
pub async fn check_unused_pillars(
    txn: &DatabaseTransaction,
    pillars: &[MazePoint],
) -> Result<Vec<unused_start_points_view::Model>, Box<dyn std::error::Error>> {
    if pillars.is_empty() {
        return Ok(Vec::new());
    }

    // 1) TEMP_UNUSED_START_POINTS から該当する柱の cell_id を取得
    let mut temp_condition = Condition::any();
    for pillar in pillars {
        temp_condition = temp_condition.add(
            Condition::all()
                .add(temp_unused_start_points::Column::X.eq(pillar.x()))
                .add(temp_unused_start_points::Column::Y.eq(pillar.y())),
        );
    }
    let temp_points = temp_unused_start_points::Entity::find()
        .filter(temp_condition)
        .all(txn)
        .await?;

    if temp_points.is_empty() {
        debug!(
            "Pillars: {:?} Checked unused pillars: none (temp cache miss)",
            pillars
        );
        return Ok(Vec::new());
    }

    // 2) cell_id で unused_start_points_view を検索
    let mut view_condition = Condition::any();
    for temp_point in temp_points {
        view_condition =
            view_condition.add(unused_start_points_view::Column::CellId.eq(temp_point.cell_id));
    }
    let unused_points: Vec<unused_start_points_view::Model> =
        unused_start_points_view::Entity::find()
            .filter(view_condition)
            .all(txn)
            .await?;
    debug!(
        "Pillars: {:?} Checked unused pillars: {:?}",
        pillars, unused_points
    );
    Ok(unused_points)
}

/// 指定された使用中セルのステータス情報を取得します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `cell` - ステータスを取得するセルの座標
/// # 戻り値
/// 使用中セルの所有者ビューのモデル、またはエラー
pub async fn get_used_cell_status(
    txn: &DatabaseTransaction,
    cell: &MazePoint,
) -> Result<maze_cell_owner_view::Model, Box<dyn std::error::Error>> {
    let used_cell_status: Vec<maze_cell_owner_view::Model> = maze_cell_owner_view::Entity::find()
        .filter(
            maze_cell_owner_view::Column::X
                .eq(cell.x())
                .and(maze_cell_owner_view::Column::Y.eq(cell.y())),
        )
        .all(txn)
        .await?;
    Ok(used_cell_status
        .into_iter()
        .next()
        .ok_or("Cell owner status not found")?)
}

/// 指定されたセルのステータス情報を取得します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `cell` - ステータスを取得するセルの座標
///
/// # 戻り値
/// セルステータスビューのモデル、またはエラー
pub async fn get_cell_status(
    txn: &DatabaseTransaction,
    cell: &MazePoint,
) -> Result<maze_cell_status_view::Model, Box<dyn std::error::Error>> {
    let cell_status: Vec<maze_cell_status_view::Model> = maze_cell_status_view::Entity::find()
        .filter(
            maze_cell_status_view::Column::X
                .eq(cell.x())
                .and(maze_cell_status_view::Column::Y.eq(cell.y())),
        )
        .all(txn)
        .await?;
    Ok(cell_status
        .into_iter()
        .next()
        .ok_or("Cell status not found")?)
}

/// 指定されたスレッドが外壁に接触しているものかどうかを判定します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `tid` - チェックするスレッドの識別子
///
/// # 戻り値
/// 外壁に接触している場合は`true`、そうでない場合は`false`とスレッドID
pub async fn is_this_thread_connect_outside_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<(bool, u64), Box<dyn std::error::Error>> {
    // THREAD_LISTから、MazeThreadIdentifierの内容に当てはまるレコードを取得する。
    let thread_record =
        select_my_thread_record_from_tx(txn, tid.thread_id_as_str(), tid.unix_time()).await?;
    if thread_record.outside_wall_connect_type
        == OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()
    {
        return Ok((false, thread_record.id));
    } else {
        return Ok((true, thread_record.id));
    }
}

/// 指定された座標が外壁の開始点かどうかを判定します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `pillar` - チェックする座標
///
/// # 戻り値
/// 外壁の開始点の場合は`true`、そうでない場合は`false`
pub async fn is_point_outside_wall(
    txn: &DatabaseTransaction,
    pillar: &MazePoint,
) -> Result<bool, Box<dyn std::error::Error>> {
    // 100分の1の確率でTEMP_OUTSIDE_WALL_START_POINTSを再生成する。(外壁の座標は初期化の際に決定されるので通常は変化しない。)
    if get_late_10_percent_flag() && get_late_10_percent_flag() {
        let txn_populate = txn.begin().await?;
        populate_temp_outside_wall_start_points(&txn_populate).await?;
        txn_populate.commit().await?;
    }
    loop {
        let txn_inner = txn.begin().await?;
        // TEMP_OUTSIDE_WALL_START_POINTSに、pillarの内容に(X AND Y)が当てはまるレコードが存在するか確認する。
        let temp_outside_res = temp_outside_wall_start_points::Entity::find()
            .filter(
                temp_outside_wall_start_points::Column::X
                    .eq(pillar.x())
                    .and(temp_outside_wall_start_points::Column::Y.eq(pillar.y())),
            )
            .one(&txn_inner)
            .await;
        let temp_outside = match temp_outside_res {
            Ok(record) => record,
            Err(e) => {
                debug!(
                    "Database error when checking outside wall start points: {}",
                    e
                );
                populate_temp_outside_wall_start_points(&txn_inner).await?;
                txn_inner.commit().await?;
                continue;
            }
        };
        txn_inner.commit().await?;
        if temp_outside.is_some() {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}

/// 指定されたセルIDの柱を取得し、スレッドの所有権を設定します。
///
/// cell_idに対応するMAZE_FIELDのCELL_TYPEがPILLARかつCELL_OWNER_THREAD_IDがNullの場合に限り、
/// CELL_OWNER_THREAD_IDをthread_idに更新します。
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `cell_id` - 取得する柱のセルID
/// * `thread_id` - 柱を所有するスレッドのID
///
/// # 戻り値
/// 更新されたMAZE_FIELDのモデル、またはエラー
///
/// # エラー
/// 更新後のレコードが見つからない場合、またはデータベースエラーが発生した場合
pub async fn get_pillar(
    txn: &DatabaseTransaction,
    cell_id: u64,
    thread_id: u64,
) -> Result<maze_field::Model, Box<dyn std::error::Error>> {
    debug!(
        "Attempting to acquire pillar cell_id={} for thread_id={}",
        cell_id, thread_id
    );
    // MAZE_FIELDのセルIDがcell_idであるレコードを取得する。
    let pillar_record: maze_field::Model = maze_field::Entity::find()
        .filter(
            maze_field::Column::Id.eq(cell_id).and(
                maze_field::Column::CellType
                    .eq("PILLAR")
                    .and(maze_field::Column::CellOwnerThreadId.is_null()),
            ),
        )
        .one(txn)
        .await?
        .ok_or(format!(
            "Pillar not found or already owned. cell_id={}",
            cell_id
        ))?;
    // CELL_OWNER_THREAD_IDをthread_idに更新する。
    let mut active_pillar = pillar_record.clone().into_active_model();
    active_pillar.cell_owner_thread_id = sea_orm::Set(Some(thread_id));
    let updated_record = maze_field::Entity::update(active_pillar).exec(txn).await?;
    debug!(
        "Pillar cell_id={} ownership updated to thread_id={}",
        cell_id, thread_id
    );
    Ok(updated_record)
}
