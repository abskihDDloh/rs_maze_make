use std::sync::{Arc, RwLock};

use log::{debug, info, warn};

use crate::{
    maze_field::{MazePoints, extend_pillar_to_adjacent_pillar, select_start_pillar_point},
    maze_point_status::WallIdentifier,
};

pub fn maze_thread_func(
    maze_points: &Arc<RwLock<MazePoints>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let identifier = WallIdentifier::new();

    loop {
        // all_pillar_seeked_flag()がtrueの場合は、全ての柱が探索済みであるため、スレッドを終了する。
        // 修正: writeロックではなくreadロックを使用
        let maze_points_read = maze_points.read().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire read lock. {:?}",
                identifier
            ))) as Box<dyn std::error::Error>
        })?;
        if maze_points_read.all_pillar_seeked_flag() {
            info!(
                "All pillars have been sought. Exiting thread. {:?}",
                identifier
            );
            return Ok(());
        }
        drop(maze_points_read); // 明示的にロックを解除

        let mut pillar_stack = Vec::new();

        // 最初の開始点を取得
        match select_start_pillar_point(maze_points, identifier.clone()) {
            Ok(pillar_point) => {
                pillar_stack.push(pillar_point);
            }
            Err(err) => {
                warn!(
                    "Failed to select start pillar point. Pillars: {:?} {}",
                    pillar_stack, err
                );
                continue;
            }
        }
        loop {
            // pillar_stackの最後の要素を取得する。
            if let Some(current_pillar) = pillar_stack.last() {
                // 修正: 引数の型を正しく渡す
                match extend_pillar_to_adjacent_pillar(
                    maze_points,
                    current_pillar.clone(), // &MazePoint → MazePoint
                    identifier.clone(),     // &WallIdentifier → WallIdentifier
                ) {
                    Ok(new_point) => {
                        if new_point.is_next_pillar() {
                            pillar_stack.push(new_point.point);
                        } else {
                            debug!(
                                "No more pillars to process. Try get other start point. {:?} Pillars: {:?}",
                                identifier, pillar_stack
                            );
                            break;
                        }
                    }
                    Err(err) => {
                        warn!("Failed to extend. Pillars: {:?} {}", pillar_stack, err);
                        break;
                    }
                }
            } else {
                debug!(
                    "No more pillars to process. Try get other start point. {:?} Pillars: {:?}",
                    identifier, pillar_stack
                );
                break;
            }
        }
    }
}
