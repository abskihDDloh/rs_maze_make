use crate::maze::maze_cell::maze_point::point::MazePoint;
use crate::maze::maze_cell::maze_point::point_status::MazePointStatus;
use crate::set_start_and_goal::path_connectivity_graph::PathConnectivityGraph;
use crate::set_start_and_goal::start_and_goal_point::StartAndGoalPoint;
use log::debug;
use petgraph::algo::astar;
use std::collections::HashMap;

/// スタートからゴールまでの経路を探索し、経路上のMazePointStatusをRESOLVED_PATHに書き換える
///
/// # 引数
/// - maze_points: 迷路の座標と状態の可変参照
/// - start_and_goal: スタート・ゴール座標を保持する構造体
///
/// # 戻り値
/// - Ok(経路のMazePointリスト) または Err(エラー)
pub fn resolve_path_from_start_to_goal(
    maze_points: &mut HashMap<MazePoint, MazePointStatus>,
    start_and_goal: &StartAndGoalPoint,
) -> Result<Vec<MazePoint>, String> {
    let start = start_and_goal.start;
    let goal = start_and_goal.goal;

    // スタート・ゴールのPathでなければエラー
    if !matches!(maze_points.get(&start), Some(s) if s.is_start_or_end_path()) {
        return Err(format!(
            "Start point ({:?}) is not a StartOrEnd path",
            start
        ));
    }
    if !matches!(maze_points.get(&goal), Some(g) if g.is_start_or_end_path()) {
        return Err(format!("Goal point ({:?}) is not a StartOrEnd path", goal));
    }

    // グラフ生成
    let graph = PathConnectivityGraph::build_from_maze_points(maze_points)
        .map_err(|e| format!("Failed to build graph: {}", e))?;

    // MazePoint→NodeIndexのマップ
    let point_to_node = &graph.point_to_node;
    let node_to_point = &graph.node_to_point;
    let start_node = *point_to_node.get(&start).ok_or("Start node not found")?;
    let goal_node = *point_to_node.get(&goal).ok_or("Goal node not found")?;

    // A*で最短経路探索（コストは全て1）
    let result = astar(&graph.graph, start_node, |n| n == goal_node, |_| 1, |_| 0);
    let path_nodes = result.ok_or("No path found from start to goal")?.1;
    // NodeIndex列をMazePoint列に変換
    let path_points: Vec<MazePoint> = path_nodes
        .into_iter()
        .filter_map(|node| node_to_point.get(&node).copied())
        .collect();

    // スタート・ゴール以外の経路上のMazePointStatusを書き換え
    for p in &path_points {
        let status = maze_points
            .get(p)
            .ok_or(format!("MazePoint {:?} not found in maze_points", p))?;
        debug!("Resolving path point: {:?}, Status: {:?}", p, status);
        if !status.is_start_or_end_path() {
            maze_points.insert(*p, MazePointStatus::new_resolved_path());
        }
    }
    Ok(path_points)
}
