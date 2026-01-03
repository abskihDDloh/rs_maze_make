use log::{Level, info, log_enabled, warn};
use rand::Rng;
use sea_orm::{ConnectionTrait, DatabaseTransaction, EntityTrait};

use crate::maze::maze_point::select_between_points_without_edge;

use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewMethodEnforcer {}

impl NewMethodEnforcer {
    fn new() -> Self {
        NewMethodEnforcer {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdatePathResult {
    MOVE_NEXT(MazePoint, NewMethodEnforcer),
    CONNECT_OUTSIDE_AND_EXIT(MazePoint, NewMethodEnforcer),
}
impl UpdatePathResult {
    fn move_next(pillar: MazePoint) -> Self {
        UpdatePathResult::MOVE_NEXT(pillar, NewMethodEnforcer::new())
    }
    fn connect_outside_and_exit(pillar: MazePoint) -> Self {
        UpdatePathResult::CONNECT_OUTSIDE_AND_EXIT(pillar, NewMethodEnforcer::new())
    }
    pub fn get_pillar(&self) -> MazePoint {
        match self {
            UpdatePathResult::MOVE_NEXT(pillar, _enforcer) => *pillar,
            UpdatePathResult::CONNECT_OUTSIDE_AND_EXIT(pillar, _enforcer) => *pillar,
        }
    }
}
macro_rules! debug_get_adjacent_extendable_pillar {
    ($tid:expr, $current_pillar:expr, $adjacent_pillars:expr) => {
        format!(
            "[tid: {:?}, current_pillar: {:?}, adjacent_pillars: {:?}]",
            $tid, $current_pillar, $adjacent_pillars
        )
    };
}
pub async fn get_adjacent_extendable_pillar(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
) -> Result<UpdatePathResult, Box<dyn std::error::Error>> {
    let mut adjacent_pillars = current_pillar.generate_adjacent_maze_points(2);

    // MAZE_CELL_MIN_MAX_VIEWの内容を取得する。
    let cell_min_max_records: Vec<crate::database::entities::maze_cell_min_max_view::Model> =
        crate::database::entities::maze_cell_min_max_view::Entity::find()
            .all(txn)
            .await?;
    if cell_min_max_records.is_empty() {
        return Err("MAZE_CELL_MIN_MAX_VIEW is empty.".into());
    }
    let max_x = cell_min_max_records[0].max_x as u64;
    let max_y = cell_min_max_records[0].max_y as u64;

    loop {
        if adjacent_pillars.is_empty() {
            return Err("No extendable adjacent pillars available.".into());
        }
        if log_enabled!(Level::Debug) {
            info!(
                "{}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, adjacent_pillars)
            );
        }
        //adjacent_pillarsの中からランダムで1個選択する。
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..adjacent_pillars.len());
        let mut selected_pillar: MazePoint = adjacent_pillars[random_index];
        //選択した要素は取り除く。
        adjacent_pillars.remove(random_index);
        if log_enabled!(Level::Debug) {
            info!(
                "{} Removed_pillar: {:?}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, adjacent_pillars),
                selected_pillar
            );
        }
        //選択した要素がMAZE_FIELDの範囲外であれば、continueする。
        if selected_pillar.x() > max_x || selected_pillar.y() > max_y {
            if log_enabled!(Level::Debug) {
                info!(
                    "{} Removed_pillar: {:?} Overflowed maze field range. max_x={}, max_y={}",
                    debug_get_adjacent_extendable_pillar!(tid, current_pillar, adjacent_pillars),
                    selected_pillar,
                    max_x,
                    max_y
                );
            }
            continue;
        }
        //GET_START_POINTプロシージャを使用して、選択した要素を取得する。
        let sql = "CALL GET_START_POINT(?, ? , ? , ?)";
        let rows_result = txn
            .query_all(sea_orm::Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                sql,
                vec![
                    selected_pillar.x().into(),
                    selected_pillar.y().into(),
                    tid.as_str().into(),
                    tid.unix_time_as_datetime_formatted_str().unwrap_or_default().into(),
                ],
            ))
            .await;

        // エラーの場合は次の候補を試す
        let rows = match rows_result {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    "GET_START_POINT procedure error for pillar ({}, {}): {:?}",
                    selected_pillar.x(),
                    selected_pillar.y(),
                    e
                );
                continue;
            } // プロシージャがエラーを返した場合は次の候補へ
        };
        //RESULT_STATUS=UPDATEDの場合は、選択した要素を返す。
        if !rows.is_empty() {
            // インデックス0でRESULT_STATUSカラムを取得
            let result_status: String = rows[0].try_get_by_index(0)?;
            if result_status == "UPDATED" {
                return Ok(UpdatePathResult::move_next(selected_pillar));
            } else if result_status == "NOT_UPDATED" {
                return Ok(UpdatePathResult::connect_outside_and_exit(selected_pillar));
            }
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
                tid.as_str().into(),
                tid.unix_time_as_datetime_formatted_str().unwrap_or_default().into(),
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

        // USED_START_POINTS_VIEWに(2,2)のレコードがあるか確認
        let selected_pillar = match result.unwrap() {
            UpdatePathResult::MOVE_NEXT(pillar, _enforcer) => pillar,
            UpdatePathResult::CONNECT_OUTSIDE_AND_EXIT(pillar, _enforcer) => pillar,
        };
        let used_points: Vec<used_start_points_view::Model> =
            entities::used_start_points_view::Entity::find()
                .filter(
                    used_start_points_view::Column::X
                        .eq(selected_pillar.x() as u64)
                        .and(used_start_points_view::Column::Y.eq(selected_pillar.y() as u64)),
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
