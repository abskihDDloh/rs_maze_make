use crate::maze::{
    independent_transaction_routines::{
        check_extendable_pillar_existance, select_random_start_point_from_db,
    },
    maze_point::MazePoint,
    maze_thread_identifier::MazeThreadIdentifier,
    move_next::{ get_adjacent_extendable_pillar, path_to_wall},
};
use log::{Level, debug, log_enabled, warn};
use sea_orm::TransactionTrait;

macro_rules! debug_maze_state {
    ($maze_stack:expr, $current_pillar:expr, $tid:expr) => {
        format!(
            "[tid: {:?}, , current_pillar: {:?}, maze_stack: {:?}]",
            $tid, $current_pillar, $maze_stack
        )
    };
}

pub async fn maze_thread_function(db: &sea_orm::DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let tid = MazeThreadIdentifier::new();
    // 未使用のスタートポイントがある場合のループ。
    loop {
        if !check_extendable_pillar_existance(db).await? {
            debug!("check_extendable_pillar_existance() = false break.");
            break;
        }
        let mut maze_stack: Vec<MazePoint> = Vec::new();
        maze_stack.push(select_random_start_point_from_db(db, &tid).await?);
        loop {
            let current_pillar = match maze_stack.last() {
                Some(p) => *p,
                None => break,
            };
            debug!(
                "Loop start. {}",
                debug_maze_state!(maze_stack, Some(current_pillar), &tid)
            );
            let txn = db.begin().await?;
            let result_get_adjacent_pillar =
                get_adjacent_extendable_pillar(&txn, &tid, &current_pillar).await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{database::initializer::WALL, maze::maze_thread::maze_thread_function};

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    #[tokio::test]
    #[test_log::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_maze_thread_completes_successfully() {
        // DB接続を確立
        let db = crate::database::connector::establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        crate::database::initializer::initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // maze_thread を実行
        let result: Result<(), Box<dyn Error>> = maze_thread_function(&db).await;
        assert!(result.is_ok(), "maze_thread failed: {:?}", result.err());

        // 条件A: UNUSED_START_POINTS_VIEW が 0 件であること
        let unused_count: u64 = crate::database::entities::unused_start_points_view::Entity::find()
            .all(&db)
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
            let cell: Vec<crate::database::entities::maze_cell_status_view::Model> =
                crate::database::entities::maze_cell_status_view::Entity::find()
                    .filter(
                        crate::database::entities::maze_cell_status_view::Column::X
                            .eq(*x)
                            .and(
                                crate::database::entities::maze_cell_status_view::Column::Y.eq(*y),
                            ),
                    )
                    .all(&db)
                    .await
                    .expect("Failed to query MAZE_CELL_STATUS_VIEW");

            if !cell.is_empty() && cell[0].cell_type == WALL {
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
            let cell: Vec<crate::database::entities::maze_cell_status_view::Model> =
                crate::database::entities::maze_cell_status_view::Entity::find()
                    .filter(
                        crate::database::entities::maze_cell_status_view::Column::X
                            .eq(*x)
                            .and(
                                crate::database::entities::maze_cell_status_view::Column::Y.eq(*y),
                            ),
                    )
                    .all(&db)
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
