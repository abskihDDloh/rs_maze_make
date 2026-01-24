use std::collections::HashSet;

use crate::{
    database::initializer::OutsideWallConnectTypeEnum,
    maze::{
        self,
        connect_to_outside_wall::get_all_adjacent_not_myself_pillar,
        independent_transaction_routines::{
            check_extendable_pillar_existance, select_random_start_point_from_db,
        },
        maze_point::MazePoint,
        maze_thread_identifier::MazeThreadIdentifier,
        move_next::{get_adjacent_unused_extendable_pillar, path_to_wall},
    },
};
use log::{debug, info, warn};
use rand::seq::SliceRandom;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

macro_rules! debug_maze_state {
    ($maze_stack:expr, $current_pillar:expr, $tid:expr) => {
        format!(
            "[tid: {:?}, , current_pillar: {:?}, maze_stack: {:?}]",
            $tid, $current_pillar, $maze_stack
        )
    };
}

pub async fn maze_thread_function(db: &sea_orm::DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let mut tids: HashSet<MazeThreadIdentifier> = HashSet::new();
    // 未使用のスタートポイントがある場合のループ。

    loop {
        let mut maze_stack: Vec<MazePoint> = Vec::new();
        let tid = MazeThreadIdentifier::new();
        if !check_extendable_pillar_existance(db).await? {
            debug!("check_extendable_pillar_existance() = false break.");
            break;
        }
        tids.insert(tid.clone());
        maze_stack.push(select_random_start_point_from_db(db, &tid).await?);
        loop {
            let current_pillar = match maze_stack.last() {
                Some(p) => *p,
                None => {
                    warn!(
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
            let txn_unused_point = db.begin().await?;
            let next_pillar_result =
                get_adjacent_unused_extendable_pillar(&txn_unused_point, &tid, &current_pillar)
                    .await;
            // エラーがあった場合はcontinueで再試行
            let next_pillar = match next_pillar_result {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "get_adjacent_unused_extendable_pillar() error. Pop current pillar and retry. : {:?}. {}",
                        e,
                        debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                    );
                    maze_stack.pop();
                    // スタックが空になった場合はコミットして内側のループを抜ける(すでに拡張可能な柱がないため)。
                    if maze_stack.is_empty() {
                        warn!(
                            "Maze stack is empty after pop, breaking inner loop. {}",
                            debug_maze_state!(maze_stack, "None", &tid)
                        );
                        txn_unused_point.commit().await?;
                        break;
                    } else {
                        txn_unused_point.rollback().await?;
                        continue;
                    }
                }
            };
            let result_to_wall =
                path_to_wall(&txn_unused_point, &tid, &current_pillar, &next_pillar).await;
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
                    txn_unused_point.rollback().await?;
                    continue;
                }
            };
            maze_stack.push(next_pillar);
            txn_unused_point.commit().await?;
        }
    }

    debug!(
        "All maze threads completed, attempting to connect to outside walls. tids: {:?}",
        tids
    );

    //tidsの中身をイテレーションしてループする。
    for tid in tids {
        // このスレッドの作った壁がすでに外壁に接続している場合は次のtidを確認する。
        let txn_check_outside_thread = db.begin().await?;
        let is_from_outside =
            crate::maze::maze_thread_utility::is_this_thread_connect_outside_wall(
                &txn_check_outside_thread,
                &tid,
            )
            .await;
        // エラーが出た場合はロールバックして次のtidへ。
        let is_from_outside = match is_from_outside {
            Ok(b) => b,
            Err(e) => {
                warn!(
                    "Failed to check outside wall connection for tid {:?}: {:?}.",
                    &tid, e
                );
                txn_check_outside_thread.rollback().await?;
                continue;
            }
        };
        txn_check_outside_thread.commit().await?;
        if is_from_outside.0 {
            info!(
                "Thread {:?} did start from outside wall, restarting maze thread.",
                &tid
            );
            continue;
        }
        info!(
            "Thread {:?} did NOT start from outside wall, finalizing maze.",
            &tid
        );
        let txn_get_adjacents = db.begin().await?;
        let adjacent_other_thread_pillars =
            get_all_adjacent_not_myself_pillar(&txn_get_adjacents, &tid).await;
        // エラーが出た場合はロールバックして次のtidへ。
        let adjacent_other_thread_pillars = match adjacent_other_thread_pillars {
            Ok(map) => map,
            Err(e) => {
                warn!(
                    "Failed to get adjacent other thread pillars for tid {:?}: {:?}.",
                    &tid, e
                );
                txn_get_adjacents.rollback().await?;
                continue;
            }
        };
        if adjacent_other_thread_pillars.is_empty() {
            warn!(
                "No adjacent other thread pillars found for tid {:?}, cannot connect to outside wall.",
                &tid
            );
            // 見つからない場合は次のtidへ。
            txn_get_adjacents.commit().await?;
            continue;
        }
        txn_get_adjacents.commit().await?;

        // adjacent_other_thread_pillarsのキーのリストを取得する。
        let mut adjacent_keys: Vec<&MazePoint> = adjacent_other_thread_pillars.keys().collect();
        // キーの内容をランダムに１つ取得する。
        let mut rng = rand::rng();
        adjacent_keys.shuffle(&mut rng);
        let selected_pillar = adjacent_keys[0];
        debug!(
            "AAAA_Selected pillar to connect outside wall: {:?}",
            selected_pillar
        );
        // selected_pillarに隣接する外壁に接続している柱セルのリストを取得する。
        let adjacent_outside_wall_pillars = &adjacent_other_thread_pillars[selected_pillar];
        // adjacent_outside_wall_pillarsの内容をランダムに１つ取得する。
        let mut rng = rand::rng();
        let mut shuffled_adjacent = adjacent_outside_wall_pillars.clone();
        shuffled_adjacent.shuffle(&mut rng);
        let outside_wall_pillar = &shuffled_adjacent[0];
        debug!(
            "AAAA_Selected outside wall pillar to connect: {:?}",
            outside_wall_pillar
        );

        let txn_finalize = db.begin().await?;
        // selected_pillarからoutside_wall_pillarまでpath_to_wall()で接続する。
        let result_to_outside_wall =
            path_to_wall(&txn_finalize, &tid, selected_pillar, outside_wall_pillar).await;
        match result_to_outside_wall {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "path_to_wall() to outside wall error for tid {:?}: {:?}.",
                    &tid, e
                );
                txn_finalize.rollback().await?;
                continue;
            }
        }
        debug!(
            "AAAA_Successfully connected thread {:?} to outside wall. Selected pillar: {:?}, Outside wall pillar: {:?}",
            &tid, selected_pillar, outside_wall_pillar
        );
        // THREAD_LISTテーブルの、tidに該当するレコードのOUTSIDE_WALL_CONNECT_TYPEの値をINDIRECT_CONNECTに変更する。
        let mut update_model = crate::database::entities::thread_list::ActiveModel {
            id: sea_orm::ActiveValue::Unchanged(is_from_outside.1),
            thread_id: sea_orm::ActiveValue::Unchanged(tid.thread_id_as_str().to_string()),
            create_unixtime: sea_orm::ActiveValue::Unchanged(tid.unix_time()),
            outside_wall_connect_type: sea_orm::ActiveValue::Set(
                OutsideWallConnectTypeEnum::INDIRECT_CONNECT.to_string(),
            ),
            ..Default::default()
        };
        let update_result = crate::database::entities::thread_list::Entity::update(update_model)
            .exec(&txn_finalize)
            .await;
        // エラーが出た場合はロールバックして次のtidへ。
        match update_result {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "AAAA_Failed to update OUTSIDE_WALL_CONNECT_TYPE for tid {:?}: {:?}.",
                    &tid, e
                );
                txn_finalize.rollback().await?;
                continue;
            }
        }
        debug!(
            "AAAA_Successfully updated thread record for tid {:?} to INDIRECT_CONNECT. Selected pillar: {:?}, Outside wall pillar: {:?}",
            &tid, selected_pillar, outside_wall_pillar
        );
        txn_finalize.commit().await?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        database::initializer::MazeCellTypeEnum::WALL, maze::maze_thread::maze_thread_function,
    };

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
            let cell: Vec<crate::database::entities::maze_cell_status_view::Model> =
                crate::database::entities::maze_cell_status_view::Entity::find()
                    .filter(
                        crate::database::entities::maze_cell_status_view::Column::X
                            .eq(*x)
                            .and(
                                crate::database::entities::maze_cell_status_view::Column::Y.eq(*y),
                            ),
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
            let cell: Vec<crate::database::entities::maze_cell_status_view::Model> =
                crate::database::entities::maze_cell_status_view::Entity::find()
                    .filter(
                        crate::database::entities::maze_cell_status_view::Column::X
                            .eq(*x)
                            .and(
                                crate::database::entities::maze_cell_status_view::Column::Y.eq(*y),
                            ),
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
