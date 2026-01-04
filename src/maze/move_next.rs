use log::{Level, debug, info, log_enabled, warn};
use rand::Rng;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, QueryFilter};

use crate::database::initializer::{DIRECT_CONNECT, NOT_CONNECT, PATH, WALL};
use crate::maze::maze_point::select_between_points_without_edge;

use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

macro_rules! debug_get_adjacent_extendable_pillar {
    ($tid:expr, $current_pillar:expr, $unused_points:expr) => {
        format!(
            "[tid: {:?}, current_pillar: {:?}, unused_points: {:?}]",
            $tid, $current_pillar, $unused_points
        )
    };
}

pub async fn get_thread_record(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<crate::database::entities::thread_list::Model, Box<dyn std::error::Error>> {
    let thread_record = crate::database::entities::thread_list::Entity::find()
        .filter(
            crate::database::entities::thread_list::Column::ThreadId
                .eq(tid.thread_id_as_str())
                .and(
                    crate::database::entities::thread_list::Column::CreateUnixtime
                        .eq(tid.unix_time()),
                ),
        )
        .one(txn)
        .await?
        .ok_or("Thread record not found.")?;
    Ok(thread_record)
}

pub async fn get_unused_pillars(
    txn: &DatabaseTransaction,
    pillars: &Vec<MazePoint>,
) -> Result<
    Vec<crate::database::entities::unused_start_points_view::Model>,
    Box<dyn std::error::Error>,
> {
    let unused_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        crate::database::entities::unused_start_points_view::Entity::find()
            .filter(
                crate::database::entities::unused_start_points_view::Column::X
                    .is_in(pillars.iter().map(|p| p.x()).collect::<Vec<u64>>())
                    .and(
                        crate::database::entities::unused_start_points_view::Column::Y
                            .is_in(pillars.iter().map(|p| p.y()).collect::<Vec<u64>>()),
                    ),
            )
            .all(txn)
            .await?;
    Ok(unused_points)
}

pub async fn get_cell_status(
    txn: &DatabaseTransaction,
    cell: &MazePoint,
) -> Result<Vec<crate::database::entities::maze_cell_status_view::Model>, Box<dyn std::error::Error>>
{
    let cell_status: Vec<crate::database::entities::maze_cell_status_view::Model> =
        crate::database::entities::maze_cell_status_view::Entity::find()
            .filter(
                crate::database::entities::maze_cell_status_view::Column::X
                    .eq(cell.x())
                    .and(crate::database::entities::maze_cell_status_view::Column::Y.eq(cell.y())),
            )
            .all(txn)
            .await?;
    Ok(cell_status)
}

pub async fn is_this_thread_from_outside_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
) -> Result<bool, Box<dyn std::error::Error>> {
    // THREAD_FROM_OUTSIDE_WALL_VIEWから、MazeThreadIdentifierの内容に当てはまるレコードを取得する。
    let thread_from_outside_wall_view_records: Vec<
        crate::database::entities::thread_from_outside_wall_view::Model,
    > = crate::database::entities::thread_from_outside_wall_view::Entity::find()
        .filter(
            crate::database::entities::thread_from_outside_wall_view::Column::Tid
                .eq(tid.thread_id_as_str()),
        )
        .all(txn)
        .await?;
    Ok(!thread_from_outside_wall_view_records.is_empty())
}

pub async fn is_point_outside_wall(txn: &DatabaseTransaction, pillar: &MazePoint) -> bool {
    // OUTSIDE_WALL_START_POINTS_VIEWに、pillarの内容に(X AND Y)が当てはまるレコードが存在するか確認する。
    let outside_wall_start_points: Vec<
        crate::database::entities::outside_wall_start_points_view::Model,
    > = crate::database::entities::outside_wall_start_points_view::Entity::find()
        .filter(
            crate::database::entities::outside_wall_start_points_view::Column::X
                .eq(pillar.x())
                .and(
                    crate::database::entities::outside_wall_start_points_view::Column::Y
                        .eq(pillar.y()),
                ),
        )
        .all(txn)
        .await
        .unwrap_or_default();
    !outside_wall_start_points.is_empty()
}

pub async fn get_adjacent_unused_extendable_pillar(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // THREAD_LISTから、MazeThreadIdentifierの内容(THREAD_ID,CREATE_UNIXTIME)に当てはまるレコードを取得する。ない場合はエラー。
    let thread_record = get_thread_record(txn, tid).await?;

    //THREAD_FROM_OUTSIDE_WALL_VIEWから、MazeThreadIdentifierの内容に当てはまるレコードを取得する。
    let thread_from_outside_wall = is_this_thread_from_outside_wall(txn, tid).await?;

    let adjacent_pillars = current_pillar.generate_adjacent_maze_points(2);

    //UNUSED_START_POINTS_VIEWから、adjacent_pillarsの内容に(X AND Y)が当てはまるレコードをすべて取得する。
    let mut unused_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        get_unused_pillars(txn, &adjacent_pillars).await?;

    loop {
        if unused_points.is_empty() {
            return Err("No extendable adjacent pillars available.".into());
        }
        debug!(
            "{}",
            debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points)
        );
        //unused_pointsの中からランダムで1個選択する。
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..unused_points.len());
        let mut selected_point = unused_points[random_index].clone();
        //選択した要素は取り除く。
        unused_points.remove(random_index);

        debug!(
            "{} Removed_pillar: {:?}",
            debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
            selected_point
        );

        let is_outside_wall_start_point =
            is_point_outside_wall(txn, &MazePoint::new(selected_point.x, selected_point.y)).await;

        // OUTSIDE_WALL_START_POINTS_VIEWに、選択した要素の(X AND Y)が当てはまるレコードが存在するか確認する。
        if thread_from_outside_wall && is_outside_wall_start_point {
            // 外壁から来た壁は外壁にはゆかないようにする。
            info!(
                "{} this thread start from outside wall. Selected pillar is outside wall start point, skipping: {:?}",
                debug_get_adjacent_extendable_pillar!(tid, current_pillar, unused_points),
                selected_point
            );
            continue; // 次の候補へ
        }

        //outside_wall_start_pointsが空でない場合は、THREAD_LISTのOUTSIDE_WALL_CONNECT_TYPEがNOT_CONNECTである場合に限り、tidの内容に当てはまるレコードのOUTSIDE_WALL_CONNECT_TYPEをDIRECT_CONNECTに更新する。
        if is_outside_wall_start_point && thread_record.outside_wall_connect_type == NOT_CONNECT {
            let update_model = crate::database::entities::thread_list::ActiveModel {
                id: sea_orm::ActiveValue::Set(thread_record.id),
                outside_wall_connect_type: sea_orm::ActiveValue::Set(DIRECT_CONNECT.to_string()),
                ..Default::default()
            };
            crate::database::entities::thread_list::Entity::update(update_model)
                .exec(txn)
                .await?;
        }

        // 選択した要素ともとの要素の間にあるセルを計算する。
        let selected_maze_point = MazePoint::new(selected_point.x, selected_point.y);
        let bitween_cell = select_between_points_without_edge(current_pillar, &selected_maze_point);
        if bitween_cell.len() != 1 {
            warn!(
                "Invalid number of between cells: current_pillar=({},{}) selected_point=({},{}) between_cells={:?}",
                current_pillar.x(),
                current_pillar.y(),
                selected_maze_point.x(),
                selected_maze_point.y(),
                bitween_cell
            );
            continue; // 次の候補へ
        }
        let adjacent_cells_candidate = bitween_cell[0];

        // MAZE_CELL_STATUS_VIEWから、adjacent_cells_candidateの内容に(X AND Y)が当てはまるレコードを取得する。
        let adjacent_cell_status = get_cell_status(txn, &adjacent_cells_candidate).await?;

        // 取得したレコードが未利用のPATHでなければ次の候補へ
        if adjacent_cell_status.is_empty()
            || adjacent_cell_status[0].cell_type != PATH
            || adjacent_cell_status[0].cell_owner_thread_id.is_some()
        {
            warn!(
                "Adjacent cell is not PATH: current_pillar=({},{}) selected_point=({},{}) adjacent_cell=({},{}) status={:?}",
                current_pillar.x(),
                current_pillar.y(),
                selected_maze_point.x(),
                selected_maze_point.y(),
                adjacent_cells_candidate.x(),
                adjacent_cells_candidate.y(),
                adjacent_cell_status
            );
            continue; // 次の候補へ
        }
        // selected_point.cell_id に当てはまるMAZE_FIELDのCELL_OWNER_THREAD_IDがNullの場合にかぎり、CELL_OWNER_THREAD_IDをthread_record.idに更新する。
        // エラーの場合は諦める。
        let maze_field_record = crate::database::entities::maze_field::Entity::find()
            .filter(crate::database::entities::maze_field::Column::Id.eq(selected_point.cell_id))
            .one(txn)
            .await?;
        if let Some(mut maze_field) = maze_field_record
            && maze_field.cell_owner_thread_id.is_none()
        {
            maze_field.cell_owner_thread_id = Some(thread_record.id);
            let update_model = crate::database::entities::maze_field::ActiveModel {
                id: sea_orm::ActiveValue::Set(maze_field.id),
                cell_owner_thread_id: sea_orm::ActiveValue::Set(maze_field.cell_owner_thread_id),
                ..Default::default()
            };
            crate::database::entities::maze_field::Entity::update(update_model)
                .exec(txn)
                .await?;
        }
        return Ok(selected_maze_point);
    }
}

pub async fn path_to_wall(
    txn: &DatabaseTransaction,
    tid: &MazeThreadIdentifier,
    current_pillar: &MazePoint,
    next_pillar: &MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // THREAD_LISTから、MazeThreadIdentifierの内容(THREAD_ID,CREATE_UNIXTIME)に当てはまるレコードを取得する。ない場合はエラー。
    let thread = get_thread_record(txn, tid).await?;
    let thread_id_from_table: u64 = thread.id;

    //current_pillarとnext_pillarの中間地点を計算する。
    let bitween_cells = select_between_points_without_edge(current_pillar, next_pillar);
    // 2個以上取得された場合はエラー。
    if bitween_cells.len() != 1 {
        // エラーメッセージにcurrent_pillar,next_pillar,bitween_cellsの内容を含める。
        return Err(format!("Invalid number of between cells: current_pillar=({},{}) next_pillar=({},{}) between_cells={:?}", current_pillar.x(), current_pillar.y(), next_pillar.x(), next_pillar.y(), bitween_cells).into());
    }
    let path_cell = bitween_cells[0];
    // 取得したレコードが未利用のPATHでなければエラー。
    let cell_status = get_cell_status(txn, &path_cell).await?;
    if cell_status.is_empty()
        || cell_status[0].cell_type != PATH
        || cell_status[0].cell_owner_thread_id.is_some()
    {
        return Err(format!("Between cell is not unused PATH: current_pillar=({},{}) next_pillar=({},{}) between_cell=({},{}) status={:?}",
        current_pillar.x(),
        current_pillar.y(),
        next_pillar.x(),
        next_pillar.y(),
        bitween_cells[0].x(),
        bitween_cells[0].y(),
        cell_status
    ).into());
    }

    // MAZE_FIELDのIDがcell_statusから取得したIDで、CELL_OWNER_THREAD_IDがNullで、CELL_TYPEがPATHの場合に限り、該当するレコードののCELL_TYPEをPATHからWALLに変更し、CELL_OWNER_THREAD_IDにthread_record.idを設定する。
    let update_model = crate::database::entities::maze_field::ActiveModel {
        id: sea_orm::ActiveValue::Set(cell_status[0].cell_id),
        cell_type: sea_orm::ActiveValue::Set(WALL.to_string()),
        cell_owner_thread_id: sea_orm::ActiveValue::Set(Some(thread_id_from_table)),
        ..Default::default()
    };
    let result = crate::database::entities::maze_field::Entity::update(update_model)
        .filter(
            crate::database::entities::maze_field::Column::CellOwnerThreadId
                .is_null()
                .and(crate::database::entities::maze_field::Column::CellType.eq(PATH))
                .and(crate::database::entities::maze_field::Column::Id.eq(cell_status[0].cell_id)),
        )
        .exec(txn)
        .await?;

    Ok(path_cell)
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
    #[test_log::test]
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
        let unix_time = tid.unix_time();

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

        let result = get_adjacent_unused_extendable_pillar(&txn, &tid, &current_pillar).await;
        txn.commit().await.expect("Failed to commit transaction");
        assert!(
            result.is_ok(),
            "get_adjacent_extendable_pillar should return Ok, got: {:?}",
            result
        );

        // USED_START_POINTS_VIEWに(2,2)のレコードがあるか確認
        let selected_pillar = match result.unwrap() {
            p if p.x() == 2 && p.y() == 2 => p,
            p => panic!(
                "Expected selected pillar to be (2,2), got ({},{})",
                p.x(),
                p.y()
            ),
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
        let unix_time = tid.unix_time();
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
