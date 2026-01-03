use log::debug;
use rand::Rng;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, Statement,
    TransactionTrait,
};

use crate::{
    database::initializer::NOT_CONNECT,
    maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier},
};

pub async fn check_extendable_pillar_existance(
    db: &sea_orm::DbConn,
) -> Result<bool, Box<dyn std::error::Error>> {
    // UNUSED_START_POINTS_VIEWのレコード数が0であればfalse、そうでない場合はtrueを返す。
    let unused_count: u64 = crate::database::entities::unused_start_points_view::Entity::find()
        .count(db)
        .await?;
    debug!("Unused start points count: {}", unused_count);
    Ok(unused_count > 0)
}

/// THREAD_LISTにおけるOUTSIDE_WALL_CONNECT_TYPE='NOT_CONNECT'のレコード数が0であればfalse、そうでない場合はtrueを返す。
/// これは、外壁接続タイプが「接続しない」のスレッドが存在するかどうかを確認するために使用される。
pub async fn check_not_connect_outside_wall_thread_existance(
    db: &sea_orm::DbConn,
) -> Result<bool, Box<dyn std::error::Error>> {
    let not_connect_count: u64 = crate::database::entities::thread_list::Entity::find()
        .filter(
            crate::database::entities::thread_list::Column::OutsideWallConnectType.eq(NOT_CONNECT),
        )
        .count(db)
        .await?;
    debug!(
        "Threads with OUTSIDE_WALL_CONNECT_TYPE='NOT_CONNECT' count: {}",
        not_connect_count
    );
    Ok(not_connect_count > 0)
}

// THREAD_LISTテーブルか自分のスレッドIDに対応するレコードを持ってくる。
pub async fn select_my_thread_record_from_db(
    db: &DbConn,
    tid: &MazeThreadIdentifier,
) -> Result<crate::database::entities::thread_list::Model, Box<dyn std::error::Error>> {
    let thread_record: crate::database::entities::thread_list::Model =
        crate::database::entities::thread_list::Entity::find()
            .filter(
                crate::database::entities::thread_list::Column::ThreadId.eq(tid.thread_id_as_str()),
            )
            .filter(
                crate::database::entities::thread_list::Column::CreateUnixtime.eq(tid.unix_time()),
            )
            .one(db)
            .await?
            .ok_or("Thread record not found")?;
    Ok(thread_record)
}

pub async fn select_random_start_point_from_db(
    db: &DbConn,
    tid: &MazeThreadIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    let txn = db.begin().await?;
    // UNUSED_START_POINTS_VIEWを全件取得する。
    let unused_start_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        crate::database::entities::unused_start_points_view::Entity::find()
            .all(&txn)
            .await?;
    if unused_start_points.is_empty() {
        return Err("No unused start points available.".into());
    }
    // ランダムに1件選択する。
    let mut rng = rand::rng();
    let random_index = rng.random_range(0..unused_start_points.len());
    let selected_point = &unused_start_points[random_index];
    let x = selected_point.x;
    let y = selected_point.y;
    let tid_str = tid.thread_id_as_str();
    let unix_time = tid.unix_time();
    // 選択したスタートポイントとMazeThreadIdentifierの内容をADD_NEW_THREADプロシージャを使ってTHERAD_LISTテーブルとMAZE_FIELDテーブルに登録する。
    let sql = "CALL ADD_NEW_THREAD(?, ?, ?, ?)";
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        sql,
        vec![
            x.into(),
            y.into(),
            tid_str.to_string().into(),
            unix_time.into(),
        ],
    ))
    .await?;
    txn.commit().await?;
    Ok(MazePoint::new(x, y))
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_select_random_start_point_five_times() {
        // DB接続を確立
        let db = crate::database::connector::establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        crate::database::initializer::initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // select_random_start_point_from_db を5回呼び出し
        let mut selected_points = Vec::new();
        for i in 0..5 {
            let tid = MazeThreadIdentifier::new();
            match select_random_start_point_from_db(&db, &tid).await {
                Ok(point) => {
                    eprintln!(
                        "Call {}: Successfully selected point ({}, {})",
                        i + 1,
                        point.x(),
                        point.y()
                    );
                    selected_points.push(point);
                }
                Err(e) => {
                    panic!("Call {}: Failed to select start point: {}", i + 1, e);
                }
            }
            let thread_record = select_my_thread_record_from_db(&db, &tid)
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

        // 検証：5個のポイントがすべて正常に取得できたこと
        assert_eq!(
            selected_points.len(),
            5,
            "Should have successfully selected 5 start points"
        );

        // すべての取得したポイントが有効な座標であること
        for point in &selected_points {
            assert!(point.x() < u64::MAX, "X coordinate should be valid");
            assert!(point.y() < u64::MAX, "Y coordinate should be valid");
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
