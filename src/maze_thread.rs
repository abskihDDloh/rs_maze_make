use std::sync::{Arc, RwLock};

use log::{debug, info, warn};

use crate::{
    maze_field::{MazePoints, extend_pillar_to_adjacent_pillar, select_start_pillar_point},
    maze_point_status::WallIdentifier,
};

pub fn maze_generate_thread(
    maze_points: &Arc<RwLock<MazePoints>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let identifier = WallIdentifier::new();

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

pub fn maze_generate_monitor_thread(
    maze_points: &Arc<RwLock<MazePoints>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_unix_timestamp = std::time::SystemTime::now();
    loop {
        let loop_unix_timestamp = std::time::SystemTime::now();
        let (x, y, all_pillar, extended_pillar, all_pillar_seeked) = {
            let maze_points_read = maze_points.read().map_err(|_| {
                Box::new(std::io::Error::other("Failed to acquire read lock"))
                    as Box<dyn std::error::Error>
            })?;

            (
                maze_points_read.x_size(),
                maze_points_read.y_size(),
                maze_points_read.get_all_pillar_points_clone().len(), // 修正: 正しいメソッド名
                maze_points_read.get_extended_pillar_points_clone().len(),
                maze_points_read.all_pillar_seeked_flag(),
            )
        }; // ここでロック解除
        let duration = match loop_unix_timestamp.duration_since(start_unix_timestamp) {
            Ok(dur) => dur.as_secs_f64(),
            Err(_) => 0.0,
        };
        // プログレス情報をログ出力
        info!(
            "Maze progress: {}/{}, Percentage:{}(%) Extended pillars per time {} , Pillars extended ({}x{}), Elapsed_time: {:?}",
            extended_pillar,
            all_pillar,
            (extended_pillar * 100) / all_pillar,
            extended_pillar as f64 / duration,
            x,
            y,
            duration,
        );

        if all_pillar_seeked {
            info!("All pillars have been extended. Exiting monitor thread.");
            return Ok(());
        }

        // 適切な間隔で監視
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
