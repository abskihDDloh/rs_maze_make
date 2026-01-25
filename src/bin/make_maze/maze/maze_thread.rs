use log::{debug, info, warn};

use rs_maze_maker::common::maze_point::MazePoint;
use rs_maze_maker::common::maze_thread_identifier::MazeThreadIdentifier;
use rs_maze_maker::common::util::get_end_time_and_elapsed_time;
use rs_maze_maker::common::util::get_now_unix_time;
use sea_orm::TransactionTrait;

use crate::maze::independent_transaction_routines::select_my_thread_record_from_db;
use crate::maze::independent_transaction_routines::select_random_start_point_from_db;
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
    const OPERATIONS_PER_COMMIT: u32 = 25;
    let mut thread_ID_cached_flag = false;
    loop {
        let mut maze_stack: Vec<MazePoint> = Vec::new();
        let pre_tid = MazeThreadIdentifier::new();
        let start_point_result = select_random_start_point_from_db(db, &pre_tid).await;
        let start_point = match start_point_result {
            Ok(p) => p,
            Err(e) => {
                debug!(
                    "No more unused start points available or error occurred: {:?}. Exiting maze thread. Current tid: {:?}",
                    e, pre_tid
                );
                break;
            }
        };

        // Always initialize tid to a safe default to avoid uninitialized use.
        let mut tid = pre_tid;
        if !thread_ID_cached_flag {
            let tid_rec_res = select_my_thread_record_from_db(db, &pre_tid).await;
            let tid_rec = match tid_rec_res {
                Ok(r) => {
                    thread_ID_cached_flag = true;
                    r
                }
                Err(e) => {
                    warn!(
                        "Failed to retrieve thread record for tid {:?}: {:?}. Continuing without cached thread ID.",
                        pre_tid, e
                    );
                    break;
                }
            };
            tid = match pre_tid.from_db_Record(tid_rec) {
                Ok(t) => t,
                Err(e) => {
                    warn!(
                        "Thread identifier mismatch for tid {:?}: {:?}. Continuing without cached thread ID.",
                        pre_tid, e
                    );
                    break;
                }
            };
        }

        maze_stack.push(start_point);

        let mut operation_count = 0;
        let mut txn_extend_wall = db.begin().await?;

        loop {
            let inner_loop_start_time = get_now_unix_time();
            let current_pillar = match maze_stack.last() {
                Some(p) => *p,
                None => {
                    info!(
                        "Maze stack is empty, breaking inner loop. {}",
                        debug_maze_state!(maze_stack, "None", &tid)
                    );
                    break;
                }
            };
            debug!(
                "Loop start. {}",
                debug_maze_state!(maze_stack, Some(current_pillar), &tid)
            );
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
                        info!(
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
                txn_extend_wall.commit().await?;
                txn_extend_wall = db.begin().await?;
                operation_count = 0;
                debug!(
                    "Committed batch, starting new transaction for thread {:?}",
                    tid
                );
            }
            let inner_loop_elpsed_time = get_end_time_and_elapsed_time(inner_loop_start_time);
            info!(
                "Inner loop elapsed time for thread {:?}: {:?} ns",
                tid, inner_loop_elpsed_time
            );
        }
        txn_extend_wall.commit().await?;
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
