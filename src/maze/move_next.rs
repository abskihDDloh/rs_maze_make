use rand::Rng;
use sea_orm::{ConnectionTrait, DatabaseTransaction};

use crate::maze::maze_point::{select_between_points, select_between_points_without_edge};

use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

pub async fn get_adjacent_extendable_pillar(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    let mut adjacent_pillars = current_pillar.generate_adjacent_maze_points(2);
    loop {
        if adjacent_pillars.is_empty() {
            return Err("No extendable adjacent pillars available.".into());
        }
        //adjacent_pillarsの中からランダムで1個選択する。
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..adjacent_pillars.len());
        let mut selected_pillar: MazePoint = adjacent_pillars[random_index];
        //選択した要素は取り除く。
        adjacent_pillars.remove(random_index);
        //GET_START_POINTプロシージャを使用して、選択した要素を取得する。
        let sql = "CALL GET_START_POINT(?, ? , ? , ?)";
        let result = txn
            .execute(sea_orm::Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                sql,
                vec![
                    selected_pillar.x().into(),
                    selected_pillar.y().into(),
                    tid.thread_id_as_str().into(),
                    tid.unix_time_as_date_time_utc()?.into(),
                ],
            ))
            .await?;
        //取得できた場合は、その要素を返す。
        if result.rows_affected() > 0 {
            return Ok(MazePoint::new(selected_pillar.x(), selected_pillar.y()));
        }
    }
}

pub async fn path_to_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
    next_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    //current_pillarとnext_pillarの中間地点を計算する。
    let bitween_cells = select_between_points_without_edge(current_pillar, next_pillar);
    // 2個以上取得された場合はエラー。
    if bitween_cells.len() != 1 {
        // エラーメッセージにcurrent_pillar,next_pillar,bitween_cellsの内容を含める。
        return Err(format!("Invalid number of between cells: current_pillar=({},{}) next_pillar=({},{}) between_cells={:?}", current_pillar.x(), current_pillar.y(), next_pillar.x(), next_pillar.y(), bitween_cells).into());
    }
    let path_cell = bitween_cells[0];
    // PATH_TO_WALLプロシージャを使用して、経路を壁に変更する。
    let sql = "CALL PATH_TO_WALL(?, ? , ? , ?)";
    let result = txn
        .execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            sql,
            vec![
                path_cell.x().into(),
                path_cell.y().into(),
                tid.thread_id_as_str().into(),
                tid.unix_time_as_date_time_utc()?.into(),
            ],
        ))
        .await?;
    //成功した場合は、経路のセルを返す。
    if result.rows_affected() > 0 {
        Ok(path_cell)
    } else {
        Err("Failed to change path to wall.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::entities;
    use crate::maze::maze_thread_identifier::MazeThreadIdentifier;
    use crate::{database::entities::used_start_points_view, maze::maze_point::MazePoint};
    use sea_orm::{
        ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Statement, TransactionTrait,
    };

    #[tokio::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_get_adjacent_extendable_pillar_with_used_start_points_view_check() {
        // DB接続
        let db = crate::database::connector::establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        crate::database::initializer::initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // スレッドID生成
        let tid = MazeThreadIdentifier::new();
        let tid_str = tid.thread_id_as_str();
        let unix_time = tid
            .unix_time_as_date_time_utc()
            .expect("failed to convert to datetime");

        // (2,0) を開始点としてADD_NEW_THREADストアドプロシージャで登録
        let sql = "CALL ADD_NEW_THREAD(?, ?, ?, ?)";
        db.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            sql,
            vec![
                2u64.into(),
                0u64.into(),
                tid_str.to_string().into(),
                unix_time.into(),
            ],
        ))
        .await
        .expect("Failed to execute ADD_NEW_THREAD");

        // (0,2) を current_pillar として get_adjacent_extendable_pillar をテスト
        let txn = db.begin().await.expect("Failed to begin transaction");
        let current_pillar = MazePoint::new(0, 2);

        let result = get_adjacent_extendable_pillar(&txn, &tid, &current_pillar).await;
        txn.commit().await.expect("Failed to commit transaction");
        assert!(
            result.is_ok(),
            "get_adjacent_extendable_pillar should return Ok, got: {:?}",
            result
        );
        let pillar = result.unwrap();
        assert_eq!(
            (pillar.x(), pillar.y()),
            (2, 2),
            "Returned pillar should be (2,2)"
        );

        // USED_START_POINTS_VIEWに(2,2)のレコードがあるか確認
        let used_points: Vec<used_start_points_view::Model> =
            entities::used_start_points_view::Entity::find()
                .filter(
                    used_start_points_view::Column::X
                        .eq(pillar.x() as u64)
                        .and(used_start_points_view::Column::Y.eq(pillar.y() as u64)),
                )
                .all(&db)
                .await
                .expect("Failed to query USED_START_POINTS_VIEW");

        assert!(
            !used_points.is_empty(),
            "USED_START_POINTS_VIEW should contain the (0,2) record"
        );
    }

    #[tokio::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_path_to_wall() {
        // DB接続
        let db = crate::database::connector::establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        crate::database::initializer::initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // スレッドID生成
        let tid = MazeThreadIdentifier::new();
        let tid_str = tid.thread_id_as_str();
        let unix_time = tid
            .unix_time_as_date_time_utc()
            .expect("failed to convert to datetime");

        // (2,2) を開始点としてADD_NEW_THREADストアドプロシージャで登録
        let sql = "CALL ADD_NEW_THREAD(?, ?, ?, ?)";
        db.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            sql,
            vec![
                2u64.into(),
                2u64.into(),
                tid_str.to_string().into(),
                unix_time.into(),
            ],
        ))
        .await
        .expect("Failed to execute ADD_NEW_THREAD");

        // path_to_wallをテスト: current_pillar=(2,2), next_pillar=(0,2)
        // 中間地点(1,2)が経路から壁に変更されることを確認
        let txn = db.begin().await.expect("Failed to begin transaction");
        let current_pillar = MazePoint::new(2, 2);
        let next_pillar = MazePoint::new(0, 2);

        let result = path_to_wall(&txn, &tid, &current_pillar, &next_pillar).await;
        txn.commit().await.expect("Failed to commit transaction");

        assert!(
            result.is_ok(),
            "path_to_wall should return Ok, got: {:?}",
            result
        );

        let wall_cell = result.unwrap();
        assert_eq!(
            (wall_cell.x(), wall_cell.y()),
            (1, 2),
            "Wall cell should be (1,2)"
        );

        // MAZE_CELL_STATUS_VIEWで、(1,2)のセルが壁になっているか確認
        let cell_status: Vec<entities::maze_cell_status_view::Model> =
            entities::maze_cell_status_view::Entity::find()
                .filter(
                    entities::maze_cell_status_view::Column::X
                        .eq(wall_cell.x() as u64)
                        .and(entities::maze_cell_status_view::Column::Y.eq(wall_cell.y() as u64)),
                )
                .all(&db)
                .await
                .expect("Failed to query MAZE_CELL_STATUS_VIEW");
    }
}
