use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use log::{debug, info, warn};

use crate::maze::{
    maze_cell::{
        maze_point::{point::MazePoint, point_status::MazePointStatus},
        wall::wall_identifier::WallIdentifier,
    },
    maze_field::field::Field,
    maze_maker::{extend_point_to_adjacent_pillar, select_start_point_any_source},
};

pub fn maze_generate_thread(
    maze_points: &Arc<RwLock<Field>>,
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
                "{:?} All start points have been sought. Exiting thread.",
                identifier
            );
            return Ok(());
        }
        drop(maze_points_read); // 明示的にロックを解除

        //今まで確認した柱もしくは開始点(更に進めなかったものを除く)
        let mut point_stack = Vec::new();

        // 最初の開始点を取得
        match select_start_point_any_source(maze_points, identifier) {
            Ok(next_point) => {
                point_stack.push(next_point);
                debug!("Selected start point: {:?}", next_point);
            }
            Err(err) => {
                debug!(
                    "Failed to select start pillar point. Pillars: {:?} {:?} {:?}",
                    identifier, point_stack, err
                );
                continue;
            }
        }
        loop {
            // point_stackの最後の要素を取得する。
            debug!(
                "Current point stack(before_extend): {:?} {:?}",
                point_stack, identifier
            );
            if let Some(current_point) = point_stack.last() {
                // 修正: 引数の型を正しく渡す
                match extend_point_to_adjacent_pillar(
                    maze_points,
                    *current_point, // &MazePoint → MazePoint
                    identifier,     // &WallIdentifier → WallIdentifier
                ) {
                    Ok(extend_result) => {
                        if extend_result.is_next_pillar() {
                            point_stack.push(extend_result.point().unwrap());
                        } else {
                            // 壁を分岐させたいので、囲まれた柱に到達したら一つ前の柱に戻る。
                            debug!(
                                "Current point stack(before_pop): {:?} {:?}",
                                point_stack, identifier
                            );
                            match point_stack.pop() {
                                Some(_val) => {
                                    debug!(
                                        "Current point stack(after_pop): {:?} {:?}",
                                        point_stack, identifier
                                    );
                                    continue;
                                }
                                None => {
                                    return Err(Box::new(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!(
                                            "Point stack is empty when trying to pop could not extend point. identifier: {:?}",
                                            identifier
                                        ),
                                    )));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Failed to extend. Pillars: {:?} {} {:?}",
                            point_stack, err, identifier
                        );
                        break;
                    }
                }
            } else {
                debug!(
                    "No more pillars to process. Try get other start point. {:?} Pillars: {:?} {:?}",
                    identifier, point_stack, identifier
                );
                break;
            }
        }
    }
}

#[allow(dead_code)]
fn output_maze_ascii_art(all_maze_points: HashMap<MazePoint, MazePointStatus>) -> String {
    let mut ascii_art = String::new();
    let x_size = all_maze_points.keys().map(|p| p.x()).max().unwrap_or(0) + 1;
    let y_size = all_maze_points.keys().map(|p| p.y()).max().unwrap_or(0) + 1;

    for y in 0..y_size {
        for x in 0..x_size {
            let point = MazePoint::new(x, y);
            let status = all_maze_points.get(&point).unwrap();
            ascii_art.push(if status.is_wall() { '#' } else { '.' });
        }
        ascii_art.push('\n');
    }

    ascii_art
}

pub fn maze_generate_monitor_thread(
    maze_points: &Arc<RwLock<Field>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_unix_timestamp = std::time::SystemTime::now();
    loop {
        let loop_unix_timestamp = std::time::SystemTime::now();
        let (
            x,
            y,
            _all_maze_points,
            all_start_points,
            extending_start_points,
            all_pillars,
            extending_pillars,
            all_start_point_seeked,
        ) = {
            let maze_points_read = maze_points.read().map_err(|_| {
                Box::new(std::io::Error::other("Failed to acquire read lock"))
                    as Box<dyn std::error::Error>
            })?;

            (
                maze_points_read.x_size(),
                maze_points_read.y_size(),
                maze_points_read.get_all_maze_points_clone(),
                maze_points_read.get_extend_start_point_clone().len(),
                maze_points_read.get_extending_start_points_clone().len(),
                maze_points_read.get_pillar_points_clone().len(),
                maze_points_read.get_extending_pillar_points_clone().len(),
                maze_points_read.all_pillar_seeked_flag(),
            )
        }; // ここでロック解除

        let duration = match loop_unix_timestamp.duration_since(start_unix_timestamp) {
            Ok(dur) => dur.as_secs_f64(),
            Err(_) => 0.0,
        };

        // プログレス情報をログ出力
        info!(
            "Start points:{}/{}, Percentage:{}(%), Extending start points per time:{}, Pillars:{}/{}, Percentage:{}(%), Extending pillars per time:{}, ({}x{}), Elapsed_time: {:?}",
            extending_start_points,
            all_start_points,
            (extending_start_points * 100) / all_start_points,
            extending_start_points as f64 / duration,
            extending_pillars,
            all_pillars,
            (extending_pillars * 100) / all_pillars,
            extending_pillars as f64 / duration,
            x,
            y,
            duration,
        );

        //info!("\n{}", output_maze_ascii_art(_all_maze_points));

        if all_start_point_seeked {
            info!("All start points have been extended. Exiting monitor thread.");
            return Ok(());
        }

        // 適切な間隔で監視
        std::thread::sleep(std::time::Duration::from_millis(10000));
    }
}
