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
use log::info;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
};

use rs_maze_maker::common::database::entities::maze_cell_owner_view;
use rs_maze_maker::common::database::entities::maze_cell_status_view;
use rs_maze_maker::common::database::entities::maze_field;
use rs_maze_maker::common::database::entities::outside_wall_start_points_view;
use rs_maze_maker::common::database::entities::thread_list;
use rs_maze_maker::common::database::entities::unused_start_points_view;
use rs_maze_maker::common::database::initializer::OutsideWallConnectTypeEnum;
use rs_maze_maker::common::maze_point::MazePoint;
use rs_maze_maker::common::maze_thread_identifier::MazeThreadIdentifier;

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

/// データベースから全ての未使用の柱（開始点）を取得します。
///
/// # 引数
/// * `txn` - データベーストランザクション
///
/// # 戻り値
/// 未使用開始点ビューのモデルのベクタ、またはエラー
pub async fn get_all_unused_pillars(
    txn: &DatabaseTransaction,
) -> Result<Vec<unused_start_points_view::Model>, Box<dyn std::error::Error>> {
    // UNUSED_START_POINTS_VIEWを全件取得する。
    let unused_start_points: Vec<unused_start_points_view::Model> =
        unused_start_points_view::Entity::find().all(txn).await?;
    Ok(unused_start_points)
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

    // すべての柱の条件をORで結合してシングルクエリで実行
    let mut condition = Condition::any();
    for pillar in pillars {
        condition = condition.add(
            Condition::all()
                .add(unused_start_points_view::Column::X.eq(pillar.x()))
                .add(unused_start_points_view::Column::Y.eq(pillar.y())),
        );
    }
    let unused_points: Vec<unused_start_points_view::Model> =
        unused_start_points_view::Entity::find()
            .filter(condition)
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
pub async fn is_point_outside_wall(txn: &DatabaseTransaction, pillar: &MazePoint) -> bool {
    // OUTSIDE_WALL_START_POINTS_VIEWに、pillarの内容に(X AND Y)が当てはまるレコードが存在するか確認する。
    let outside_wall_start_points: Vec<outside_wall_start_points_view::Model> =
        outside_wall_start_points_view::Entity::find()
            .filter(
                outside_wall_start_points_view::Column::X
                    .eq(pillar.x())
                    .and(outside_wall_start_points_view::Column::Y.eq(pillar.y())),
            )
            .all(txn)
            .await
            .unwrap_or_default();
    !outside_wall_start_points.is_empty()
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
