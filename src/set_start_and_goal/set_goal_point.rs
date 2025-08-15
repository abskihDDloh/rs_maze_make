use crate::maze::maze_cell::maze_point::point::MazePoint;
use crate::maze::maze_cell::maze_point::point_status::MazePointStatus;
use crate::set_start_and_goal::path_connectivity_graph::PathConnectivityGraph;
use std::collections::HashMap;

/// maze_pointsとfarthest_pointをもとに、
/// farthest_pointから最遠のMazePointをStartOrEndとしてマークし返す。
///
/// # 引数
/// * `maze_points` - 迷路の全座標点と状態
/// * `farthest_point` - 既にStartOrEndにマークされたMazePoint
///
/// # 戻り値
/// * `Ok(MazePoint)` - 新たな最遠点（StartOrEndにマーク済み）
/// * `Err(_)` - 条件不一致や探索失敗時
pub fn mark_goal_point_from_farthest(
    maze_points: &mut HashMap<MazePoint, MazePointStatus>,
    farthest_point: MazePoint,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // 1. farthest_pointがmaze_pointsに含まれているか確認
    let status = maze_points
        .get(&farthest_point)
        .ok_or("farthest_point not found in maze_points")?;
    // 2. pathかつStartOrEndであることを確認
    if !status.is_path() || !status.is_start_or_end_path() {
        return Err("farthest_point is not a StartOrEnd path".into());
    }
    // 3. PathConnectivityGraph生成
    let graph = PathConnectivityGraph::build_from_maze_points(maze_points)?;
    // 4. farthest_pointに対応するノードを取得
    let &start_node = graph
        .point_to_node
        .get(&farthest_point)
        .ok_or("farthest_point not found in graph nodes")?;
    // 5. 幅優先探索で最遠点を求める
    use petgraph::visit::Bfs;
    let mut bfs = Bfs::new(&graph.graph, start_node);
    let mut last_node = start_node;
    while let Some(node) = bfs.next(&graph.graph) {
        last_node = node;
    }
    // 6. 最遠点のMazePointを取得
    let goal_point = graph
        .node_to_point
        .get(&last_node)
        .ok_or("NodeIndex to MazePoint mapping failed")?;
    // 7. maze_pointsにStartOrEndとしてマーク
    maze_points.insert(*goal_point, MazePointStatus::new_start_or_end_path());
    Ok(*goal_point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;

    #[test]
    fn test_mark_goal_point_from_farthest_simple() {
        // 2x2のうち(0,0)と(1,0)が通路、他は壁
        let mut maze_points = HashMap::new();
        maze_points.insert(
            MazePoint::new(0, 0),
            MazePointStatus::new_start_or_end_path(),
        );
        maze_points.insert(
            MazePoint::new(1, 0),
            MazePointStatus::new_not_resolved_path(),
        );
        maze_points.insert(
            MazePoint::new(0, 1),
            MazePointStatus::new_maze_wall(WallIdentifier::new()),
        );
        maze_points.insert(
            MazePoint::new(1, 1),
            MazePointStatus::new_maze_wall(WallIdentifier::new()),
        );

        let result = mark_goal_point_from_farthest(&mut maze_points, MazePoint::new(0, 0));
        assert!(result.is_ok());
        let goal = result.unwrap();
        // goalは(1,0)でStartOrEndになっているはず
        assert_eq!(goal, MazePoint::new(1, 0));
        let status = maze_points.get(&goal).unwrap();
        assert!(status.is_start_or_end_path());
    }

    #[test]
    fn test_mark_goal_point_from_farthest_not_found() {
        // farthest_pointがmaze_pointsに存在しない場合
        let mut maze_points = HashMap::new();
        maze_points.insert(
            MazePoint::new(1, 0),
            MazePointStatus::new_not_resolved_path(),
        );
        let result = mark_goal_point_from_farthest(&mut maze_points, MazePoint::new(0, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_goal_point_from_farthest_not_start_or_end() {
        // farthest_pointがStartOrEndでない場合
        let mut maze_points = HashMap::new();
        maze_points.insert(
            MazePoint::new(0, 0),
            MazePointStatus::new_not_resolved_path(),
        );
        maze_points.insert(
            MazePoint::new(1, 0),
            MazePointStatus::new_not_resolved_path(),
        );
        let result = mark_goal_point_from_farthest(&mut maze_points, MazePoint::new(0, 0));
        assert!(result.is_err());
    }
}
