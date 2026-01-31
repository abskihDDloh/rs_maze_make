use log::{debug, info, warn};

use rs_maze_maker::common::database::initializer::populate_temp_unused_start_points;
use rs_maze_maker::common::maze_point::MazePoint;
use sea_orm::TransactionTrait;

use crate::maze::independent_transaction_routines::select_random_start_point_from_db;
use crate::maze::maze_thread_utility::get_unused_points_count_from_temp_records;
use crate::maze::move_next::get_adjacent_unused_extendable_pillar;
use crate::maze::move_next::path_to_wall;

macro_rules! debug_maze_state {
    ($maze_stack:expr, $current_pillar:expr, $tid:expr) => {
        format!(
            "[tid: {:?}, , current_pillar: {:?}, maze_stack: {:?}]",
            $tid, $current_pillar, $maze_stack
        )
    };
}

pub async fn maze_thread_function(db: &sea_orm::DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // 未使用のスタートポイントがある場合のループ。
    // ボトルネック対策: 定期的にコミットしてロックを解放
    const OPERATIONS_PER_COMMIT: u32 = 500;
    loop {
        let mut maze_stack: Vec<MazePoint> = Vec::new();

        let start_point_result = select_random_start_point_from_db(db).await;
        let start_point_information = match start_point_result {
            Ok(p) => p,
            Err(e) => {
                debug!(
                    "No more unused start points available or error occurred: {:?}. Exiting maze thread.",
                    e
                );
                break;
            }
        };

        let tid = start_point_information.1.clone();
        maze_stack.push(start_point_information.0);

        let mut operation_count = 0;
        let mut txn_extend_wall = db.begin().await?;

        loop {
            let current_pillar = match maze_stack.last() {
                Some(p) => *p,
                None => {
                    debug!(
                        "Maze stack is empty, breaking inner loop. {}",
                        debug_maze_state!(maze_stack, "None", &tid)
                    );
                    break;
                }
            };
            let next_pillar_result =
                get_adjacent_unused_extendable_pillar(&txn_extend_wall, &tid, &current_pillar)
                    .await;
            // エラーがあった場合はcontinueで再試行
            let next_pillar = match next_pillar_result {
                Ok(p) => p,
                Err(e) => {
                    debug!(
                        "get_adjacent_unused_extendable_pillar() error. Pop current pillar and retry. : {:?}. {}",
                        e,
                        debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                    );
                    maze_stack.pop();
                    // スタックが空になった場合は内側のループを抜けてコミットする(すでに拡張可能な柱がないため)。
                    if maze_stack.is_empty() {
                        debug!(
                            "Maze stack is empty after pop, breaking inner loop. {}",
                            debug_maze_state!(maze_stack, "None", &tid)
                        );
                        break;
                    } else {
                        continue;
                    }
                }
            };
            let result_to_wall = path_to_wall(
                &txn_extend_wall,
                tid.thread_id_as_str(),
                tid.unix_time(),
                &current_pillar,
                &next_pillar,
            )
            .await;
            // エラーがあった場合はcontinueで再試行
            match result_to_wall {
                Ok(_) => {}
                Err(e) => {
                    warn!(
                        "path_to_wall() error. Pop current pillar and retry. : {:?}. {}",
                        e,
                        debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                    );
                    maze_stack.pop();
                    continue;
                }
            };
            maze_stack.push(next_pillar);
            operation_count += 1;

            // 定期的にコミットしてロックを解放（ボトルネック対策）
            if operation_count >= OPERATIONS_PER_COMMIT {
                populate_temp_unused_start_points(&txn_extend_wall).await?;
                txn_extend_wall.commit().await?;
                txn_extend_wall = db.begin().await?;
                operation_count = 0;
            }
        }
        populate_temp_unused_start_points(&txn_extend_wall).await?;
        txn_extend_wall.commit().await?;
    }
    Ok(())
}

pub async fn execute_maze_threads_threads_to_outside_wall(
    db: &sea_orm::DbConn,
    thread_limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let txn_get_unused_counts = db.begin().await?;
        populate_temp_unused_start_points(&txn_get_unused_counts).await?;
        let unused_counts =
            get_unused_points_count_from_temp_records(&txn_get_unused_counts).await?;
        txn_get_unused_counts.commit().await?;
        // 未使用のスタートポイントが存在しない場合は処理を終了する。
        if unused_counts == 0 {
            info!("All maze threads have connected to outside wall. No action needed.");
            break;
        }
        // thread_limitの数だけ並列でmaze_thread_functionの処理を行う。
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async {
                let mut handles = vec![];
                for _ in 0..thread_limit {
                    let db_clone = db.clone();
                    let handle =
                        tokio::task::spawn_local(
                            async move { maze_thread_function(&db_clone).await },
                        );
                    handles.push(handle);
                }
                // すべてのタスクの完了を待つ
                for handle in handles {
                    let result = handle.await;
                    // 利用可能な開始点がなくなった時点で必ずエラーになる。
                    match result {
                        Ok(_) => info!("Maze generation task completed successfully."),
                        Err(e) => warn!("Maze generation task failed: {}", e),
                    }
                }
            })
            .await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rs_maze_maker::common::database::entities::maze_cell_status_view;
    use rs_maze_maker::common::database::entities::unused_start_points_view;
    use rs_maze_maker::common::database::initializer::MazeCellTypeEnum::WALL;
    use rs_maze_maker::common::database::{
        connector::establish_connection, initializer::initialize_db,
    };
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    use crate::maze::maze_thread::maze_thread_function;

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_maze_thread_completes_successfully() {
        // DB接続を確立
        let db = establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // maze_thread を実行
        let result: Result<(), Box<dyn Error>> = maze_thread_function(&db).await;
        assert!(result.is_ok(), "maze_thread failed: {:?}", result.err());

        // 条件A: UNUSED_START_POINTS_VIEW が 0 件であること
        let unused_count: u64 = unused_start_points_view::Entity::find()
            .all(db.as_ref())
            .await
            .expect("Failed to count unused start points")
            .len() as u64;
        assert_eq!(
            unused_count, 0,
            "UNUSED_START_POINTS_VIEW should be empty but has {} records",
            unused_count
        );

        // 条件B: 以下の座標のいずれか1つだけが "WALL" になっている
        let wall_candidates = vec![(2, 1), (3, 2), (2, 3), (1, 2)];
        let mut wall_found = false;
        let mut wall_count = 0;
        for (x, y) in &wall_candidates {
            let cell: Vec<maze_cell_status_view::Model> = maze_cell_status_view::Entity::find()
                .filter(
                    maze_cell_status_view::Column::X
                        .eq(*x)
                        .and(maze_cell_status_view::Column::Y.eq(*y)),
                )
                .all(db.as_ref())
                .await
                .expect("Failed to query MAZE_CELL_STATUS_VIEW");

            if !cell.is_empty() && cell[0].cell_type == WALL.to_string() {
                wall_found = true;
                wall_count += 1;
                eprintln!("Found WALL at ({}, {})", x, y);
                break;
            }
        }
        assert!(
            wall_found,
            "At least one of {:?} should be WALL",
            wall_candidates
        );
        assert_eq!(
            wall_count, 1,
            "Only one of {:?} should be WALL, but found {}",
            wall_candidates, wall_count
        );

        // 条件C: 以下の座標がすべて "PATH" になっている
        let path_coords = vec![(1, 1), (3, 1), (3, 3), (1, 3)];
        for (x, y) in &path_coords {
            let cell: Vec<maze_cell_status_view::Model> = maze_cell_status_view::Entity::find()
                .filter(
                    maze_cell_status_view::Column::X
                        .eq(*x)
                        .and(maze_cell_status_view::Column::Y.eq(*y)),
                )
                .all(db.as_ref())
                .await
                .expect("Failed to query MAZE_CELL_STATUS_VIEW");

            assert!(!cell.is_empty(), "Cell at ({}, {}) should exist", x, y);
            assert_eq!(
                cell[0].cell_type, "PATH",
                "Cell at ({}, {}) should be PATH but is {}",
                x, y, cell[0].cell_type
            );
            eprintln!("Verified PATH at ({}, {})", x, y);
        }

        eprintln!("All conditions satisfied!");
    }
}
