use log::debug;

use rs_maze_maker::common::database::entities::thread_list;
use rs_maze_maker::common::database::initializer::OutsideWallConnectTypeEnum;
use rs_maze_maker::common::maze_point::MazePoint;
use sea_orm::DbConn;
use sea_orm::EntityTrait;
use sea_orm::TransactionTrait;

use crate::maze::maze_thread_identifier::MazeThreadIdentifier;
use crate::maze::maze_thread_utility::get_pillar;
use crate::maze::maze_thread_utility::is_point_outside_wall;
use crate::maze::maze_thread_utility::select_random_start_point;

/// 未使用の開始ポイントをランダムに選択し、THREAD_LISTテーブルに新しいスレッドレコードを追加する。
/// # 引数
/// * `db` - データベース接続
/// # 戻り値
/// 選択された開始ポイントのMazePointと対応するMazeThreadIdentifierのタプル
pub async fn select_random_start_point_from_db(
    db: &DbConn,
) -> Result<(MazePoint, MazeThreadIdentifier), Box<dyn std::error::Error>> {
    let pre_tid = MazeThreadIdentifier::new();
    // UNUSED_START_POINTS_VIEWからランダムに1件選択する。
    let txn = db.begin().await?;
    let selected_point = select_random_start_point(&txn).await?;
    let selected_cell_id = selected_point.cell_id;
    let x = selected_point.x;
    let y = selected_point.y;
    let tid_str = pre_tid.thread_id_as_str();
    let unix_time = pre_tid.unix_time();

    let selected_maze_point = MazePoint::new(x, y);
    let is_outside = is_point_outside_wall(&txn, &selected_maze_point).await?;

    let outside_wall_connect_type = if is_outside {
        OutsideWallConnectTypeEnum::DIRECT_CONNECT
    } else {
        OutsideWallConnectTypeEnum::NOT_CONNECT
    };

    debug!(
        "Selected random start point: cell_id={}, x={}, y={}, is_outside_wall={}, outside_wall_connect_type={}",
        selected_cell_id, x, y, is_outside, outside_wall_connect_type
    );

    // THREAD_LISTテーブルに新しいスレッドレコードを追加する。
    // THREAD_ID=tid_str, CREATE_UNIXTIME=unix_time, START_CELL=selected_cell_id, OUTSIDE_WALL_CONNECT_TYPE=outside_wall_connect_type
    let result = thread_list::Entity::insert(thread_list::ActiveModel {
        thread_id: sea_orm::Set(tid_str.to_string()),
        create_unixtime: sea_orm::Set(unix_time),
        start_cell: sea_orm::Set(selected_cell_id),
        outside_wall_connect_type: sea_orm::Set(outside_wall_connect_type.to_string()),
        ..Default::default()
    })
    .exec(&txn)
    .await?;
    // 追加したスレッドレコードのID列を取得する。
    let _new_thread_id = result.last_insert_id;
    let post_tid = pre_tid.fill_info(_new_thread_id, outside_wall_connect_type.to_string());

    // selected_cell_idに当てはまるMAZE_FIELDのCELL_TYPEがPILLARかつCELL_OWNER_THREAD_IDがNullの場合にかぎり、CELL_OWNER_THREAD_IDを_new_thread_idに更新する。
    let _pillar_update_result = get_pillar(&txn, selected_cell_id, _new_thread_id).await?;

    txn.commit().await?;
    debug!(
        "Inserted new thread record with ID={}, THREAD_ID={}, CREATE_UNIXTIME={}, START_CELL={}, OUTSIDE_WALL_CONNECT_TYPE={}, Pillar update result={:?}",
        _new_thread_id,
        tid_str,
        unix_time,
        selected_cell_id,
        outside_wall_connect_type,
        _pillar_update_result
    );
    Ok((MazePoint::new(x, y), post_tid))
}

#[cfg(test)]
mod tests {
    use log::info;
    use rs_maze_maker::common::database::entities::unused_start_points_view;
    use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter};

    use crate::maze::maze_thread_utility::select_my_thread_record_from_tx;

    use super::*;
    use rs_maze_maker::common::database::connector::establish_connection;
    use rs_maze_maker::common::database::initializer::initialize_db;

    async fn check_extendable_pillar_existance(
        db: &sea_orm::DbConn,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // UNUSED_START_POINTS_VIEWのレコード数が0であればfalse、そうでない場合はtrueを返す。
        let unused_count: u64 = unused_start_points_view::Entity::find().count(db).await?;
        debug!("Unused start points count: {}", unused_count);
        Ok(unused_count > 0)
    }

    /// THREAD_LISTにおけるOUTSIDE_WALL_CONNECT_TYPE='NOT_CONNECT'のレコード数が0であればfalse、そうでない場合はtrueを返す。
    /// これは、外壁接続タイプが「接続しない」のスレッドが存在するかどうかを確認するために使用される。
    async fn check_not_connect_outside_wall_thread_existance(
        db: &sea_orm::DbConn,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let not_connect_count: u64 = thread_list::Entity::find()
            .filter(
                thread_list::Column::OutsideWallConnectType
                    .eq(OutsideWallConnectTypeEnum::NOT_CONNECT.to_string()),
            )
            .count(db)
            .await?;

        debug!(
            "Threads with OUTSIDE_WALL_CONNECT_TYPE='NOT_CONNECT' count: {}",
            not_connect_count
        );

        Ok(not_connect_count > 0)
    }

    // THREAD_LISTテーブルから自分のスレッドIDに対応するレコードを持ってくる。
    async fn select_my_thread_record_from_db(
        db: &DbConn,
        tid: &MazeThreadIdentifier,
    ) -> Result<thread_list::Model, Box<dyn std::error::Error>> {
        let txn = db.begin().await?;
        let thread_record: thread_list::Model =
            select_my_thread_record_from_tx(&txn, tid.thread_id_as_str(), tid.unix_time()).await?;
        txn.commit().await?;
        Ok(thread_record)
    }

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_select_random_start_point_five_times() {
        // DB接続を確立
        let db: std::sync::Arc<sea_orm::DatabaseConnection> = establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // select_random_start_point_from_db を5回呼び出し
        let mut selected_points = Vec::new();
        for i in 0..5 {
            match select_random_start_point_from_db(&db).await {
                Ok(point) => {
                    eprintln!(
                        "Call {}: Successfully selected point ({}, {})",
                        i + 1,
                        point.0.x(),
                        point.0.y()
                    );
                    selected_points.push(point.clone());
                    let thread_record = select_my_thread_record_from_db(&db, &point.1)
                        .await
                        .expect("Failed to retrieve thread record after selecting start point");
                    info!(
                        "Retrieved thread record: ID={}, THREAD_ID={}, CREATE_UNIXTIME={}, START_CELL={}, OUTSIDE_WALL_CONNECT_TYPE={}",
                        thread_record.id,
                        thread_record.thread_id,
                        thread_record.create_unixtime,
                        thread_record.start_cell,
                        thread_record.outside_wall_connect_type
                    );
                }
                Err(e) => {
                    panic!("Call {}: Failed to select start point: {}", i + 1, e);
                }
            }
        }

        // 検証：5個のポイントがすべて正常に取得できたこと
        assert_eq!(
            selected_points.len(),
            5,
            "Should have successfully selected 5 start points"
        );

        // すべての取得したポイントが有効な座標であること
        for point in &selected_points {
            assert!(point.0.x() < u64::MAX, "X coordinate should be valid");
            assert!(point.0.y() < u64::MAX, "Y coordinate should be valid");
        }
        let res = check_extendable_pillar_existance(&db).await;
        assert!(res.is_ok());
        let has_extendable = res.unwrap();
        assert!(
            !has_extendable,
            "There should not be extendable pillars available"
        );

        info!(
            "Successfully selected {} start points without errors",
            selected_points.len()
        );

        // check_not_connect_outside_wall_thread_existanceの結果はfalseの想定(このテストコードだと2.2がNOT_CONNTCTのままになるはず)。
        let res = check_not_connect_outside_wall_thread_existance(&db).await;
        assert!(res.is_ok());
        let has_not_connect = res.unwrap();
        assert!(
            has_not_connect,
            "There should not be threads with OUTSIDE_WALL_CONNECT_TYPE='NOT_CONNECT'"
        );
    }
}
