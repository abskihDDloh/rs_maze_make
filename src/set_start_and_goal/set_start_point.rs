use crate::maze::maze_cell::maze_point::point::MazePoint;
use crate::maze::maze_cell::maze_point::point_status::MazePointStatus;
use crate::set_start_and_goal::path_connectivity_graph::PathConnectivityGraph;
use rand::seq::IteratorRandom;
use std::collections::HashMap;

/// maze_pointsからPathConnectivityGraphを生成し、
/// ランダムなPathノードを起点に幅優先探索で最遠点を求め、
/// そのMazePointをStartOrEndとしてマークして返す。
///
/// # 引数
/// * `maze_points` - 迷路の全座標点と状態
///
/// # 戻り値
/// * `Ok(MazePoint)` - 最遠点（StartOrEndにマーク済み）
/// * `Err(_)` - グラフ生成や探索失敗時
pub fn mark_farthest_point_as_start_or_end(
    maze_points: &mut HashMap<MazePoint, MazePointStatus>,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    // 1. PathConnectivityGraph生成
    let graph = PathConnectivityGraph::build_from_maze_points(maze_points)?;
    if graph.graph.node_count() == 0 {
        return Err("No path node found".into());
    }
    // 2. ランダムなノードを選択
    let mut rng = rand::rng();

    let start_node = graph
        .graph
        .node_indices()
        .choose(&mut rng)
        .ok_or("Failed to select random node")?;

    // 3. 幅優先探索で最遠点を求める
    use petgraph::visit::Bfs;
    let mut bfs = Bfs::new(&graph.graph, start_node);
    let mut last_node = start_node;
    while let Some(node) = bfs.next(&graph.graph) {
        last_node = node;
    }
    // 4. 最遠点のMazePointを取得
    let farthest_point = graph.node_to_point.get(&last_node).ok_or(format!(
        "NodeIndex to MazePoint mapping failed. {:?}",
        last_node
    ))?;
    // 5. maze_pointsにStartOrEndとしてマーク
    maze_points.insert(*farthest_point, MazePointStatus::new_start_or_end_path());
    Ok(*farthest_point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;

    #[test]
    fn test_mark_farthest_point_as_start_or_end_simple() {
        // 2x2のうち(0,0)と(1,0)が通路、他は壁
        let mut maze_points = HashMap::new();
        maze_points.insert(
            MazePoint::new(0, 0),
            MazePointStatus::new_not_resolved_path(),
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

        let result = mark_farthest_point_as_start_or_end(&mut maze_points);
        assert!(result.is_ok());
        let farthest = result.unwrap();
        // 最遠点が通路のどちらかであること
        assert!(farthest == MazePoint::new(0, 0) || farthest == MazePoint::new(1, 0));
        // マークがStartOrEndになっていること
        let status = maze_points.get(&farthest).unwrap();
        assert!(status.is_start_or_end_path());
    }

    #[test]
    fn test_mark_farthest_point_as_start_or_end_all_wall() {
        // 全て壁の場合はエラー
        let mut maze_points = HashMap::new();
        for x in 0..2 {
            for y in 0..2 {
                maze_points.insert(
                    MazePoint::new(x, y),
                    MazePointStatus::new_maze_wall(WallIdentifier::new()),
                );
            }
        }
        let result = mark_farthest_point_as_start_or_end(&mut maze_points);
        assert!(result.is_err());
    }
}
