use crate::maze::{
    maze_point::MazePoint,
    maze_thread_identifier::MazeThreadIdentifier,
    move_next::{UpdatePathResult, get_adjacent_extendable_pillar, path_to_wall},
    start_point_selector::{check_extendable_pillar_existance, select_random_start_point_from_db},
};
use log::{Level, debug, info, log_enabled, warn};
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
    // ここに maze_thread の実装を追加してください
    loop {
        let mut maze_stack: Vec<MazePoint> = Vec::new();
        if !check_extendable_pillar_existance(db).await? {
            if log_enabled!(Level::Debug) {
                info!(
                    "check_extendable_pillar_existance() = false {}",
                    debug_maze_state!(maze_stack, "None", "None")
                );
            }
            return Ok(());
        }
        let tid = MazeThreadIdentifier::new();
        let start_point = select_random_start_point_from_db(db, &tid).await?;
        maze_stack.push(start_point);
        loop {
            if !check_extendable_pillar_existance(db).await? {
                if log_enabled!(Level::Debug) {
                    info!(
                        "check_extendable_pillar_existance() = false {}",
                        debug_maze_state!(maze_stack, "None", &tid)
                    );
                }
                return Ok(());
            }

            //maze_stackの最後の要素を取得する。
            let current_pillar = match maze_stack.last() {
                Some(p) => *p,
                None => break,
            };
            if log_enabled!(Level::Debug) {
                info!(
                    "Loop start. {}",
                    debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                );
            }

            let txn = db.begin().await?;
            let result_get_adjacent_pillar =
                get_adjacent_extendable_pillar(&txn, &tid, &current_pillar).await;
            // エラーの場合はトランザクションをコミットし、1つ前の柱に戻ってリトライする。
            if result_get_adjacent_pillar.is_err() {
                warn!(
                    "Failed to extend. {}, result: {:?}",
                    debug_maze_state!(maze_stack, Some(current_pillar), &tid),
                    result_get_adjacent_pillar
                );
                txn.commit().await?;
                maze_stack.pop();
                continue;
            }
            let unwrapped_update_path_result = result_get_adjacent_pillar.unwrap();
            // 成功した場合はUpdatePathResultの値によって処理を分岐する。

            let next_pillar = unwrapped_update_path_result.get_pillar();

            // 共通処理として、柱の間の通路を壁に変更する。
            let result_path_to_wall = path_to_wall(&txn, &tid, &current_pillar, &next_pillar).await;
            // エラーの場合はトランザクションをロールバックし、1つ前の柱に戻ってリトライする。
            if result_path_to_wall.is_err() {
                warn!(
                    "Failed to create path to wall. maze_stack: {:?} curerent_pillar: {:?}, next_pillar: {:?}, tid: {:?}, result: {:?}",
                    maze_stack, current_pillar, next_pillar, tid, result_path_to_wall
                );
                txn.rollback().await?;
                maze_stack.pop();
                continue;
            }
            match unwrapped_update_path_result {
                // MOVE_NEXTの場合はpath_to_wall()を呼び出した後  maze_stack.push()を呼び出してトランザクションをコミットし、continueする。
                UpdatePathResult::MOVE_NEXT(_, _) => {
                    txn.commit().await?;
                    maze_stack.push(next_pillar);
                    if log_enabled!(Level::Debug) {
                        info!(
                            "Loop end (continue). {}",
                            debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                        );
                    }
                    continue;
                }
                // CONNECT_OUTSIDE_AND_EXITの場合はpath_to_wall()を呼び出した後 トランザクションをコミットし、正常終了する。
                UpdatePathResult::CONNECT_OUTSIDE_AND_EXIT(_, _) => {
                    txn.commit().await?;
                    if log_enabled!(Level::Debug) {
                        info!(
                            "Loop end (return). {}",
                            debug_maze_state!(maze_stack, Some(current_pillar), &tid)
                        );
                    }
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "maze_thread_test.rs"]
mod maze_thread_test;
