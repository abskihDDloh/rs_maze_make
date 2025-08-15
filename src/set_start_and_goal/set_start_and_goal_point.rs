/// 迷路のスタート・ゴール座標を自動的に決定し、両方を返すユーティリティ関数。
///
/// # 概要
/// - 迷路の全座標点（`maze_points`）から、
///   1. 通路グラフ上でランダムな点から最遠の点を「スタート」としてマーク
///   2. そのスタート点からさらに最遠の点を「ゴール」としてマーク
/// - スタート・ゴールは`MazePointStatus::StartOrEnd`でマークされる
///
/// # 引数
/// * `maze_points` - 迷路の全座標点と状態（可変参照）
///
/// # 戻り値
/// * `Ok((start, goal))` - スタート・ゴール座標（両方`MazePoint`）
/// * `Err(_)` - 迷路が不正・経路が存在しない等
///
/// # エラー条件
/// - 通路が存在しない場合
/// - スタート・ゴールの決定に失敗した場合
///
/// # 使用例
/// ```rust
/// let mut maze_points = ...; // 迷路データを用意
/// let (start, goal) = set_start_and_goal_point(&mut maze_points)?;
/// println!("start={:?}, goal={:?}", start, goal);
/// ```
///
/// # 関連
/// - [`mark_farthest_point_as_start_or_end`] : スタート決定
/// - [`mark_goal_point_from_farthest`] : ゴール決定
///
use std::collections::HashMap;

use crate::{
    maze::maze_cell::maze_point::{point::MazePoint, point_status::MazePointStatus},
    set_start_and_goal::{
        set_goal_point::mark_goal_point_from_farthest,
        set_start_point::mark_farthest_point_as_start_or_end,
        start_and_goal_point::StartAndGoalPoint,
    },
};

pub fn set_start_and_goal_point(
    maze_points: &mut HashMap<MazePoint, MazePointStatus>,
) -> Result<StartAndGoalPoint, Box<dyn std::error::Error>> {
    let start = mark_farthest_point_as_start_or_end(maze_points)?;
    let goal = mark_goal_point_from_farthest(maze_points, start)?;
    Ok(StartAndGoalPoint { start, goal })
}
