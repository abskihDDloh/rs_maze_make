use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

use crate::maze::maze_cell::maze_point::point::MazePoint;
use crate::maze::maze_cell::maze_point::point_status::MazePointStatus;

/// Pathの連結性を分析するためのグラフ構造
#[derive(Default)]
pub struct PathConnectivityGraph {
    /// petgraphのグラフインスタンス
    pub graph: UnGraph<MazePoint, ()>,
    /// MazePointからNodeIndexへのマッピング
    pub point_to_node: HashMap<MazePoint, NodeIndex>,
    /// NodeIndexからMazePointへのマッピング
    pub node_to_point: HashMap<NodeIndex, MazePoint>,
}

impl PathConnectivityGraph {
    fn default() -> Self {
        PathConnectivityGraph {
            graph: UnGraph::new_undirected(),
            node_to_point: HashMap::new(),
            point_to_node: HashMap::new(),
        }
    }

    pub fn build_from_maze_points(
        maze_points: &HashMap<MazePoint, MazePointStatus>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // maze_pointsに含まれるMazePointの一覧を取得し、x,yの最大値を探す。
        let (x_max, y_max) = maze_points.keys().fold((0u64, 0u64), |(x_max, y_max), point| {
            (x_max.max(point.x()), y_max.max(point.y()))
        });

        let x_size = x_max + 1;
        let y_size = y_max + 1;
        let mut graph = UnGraph::new_undirected();
        let mut point_to_node: HashMap<MazePoint, NodeIndex> = HashMap::new();
        let mut node_to_point: HashMap<NodeIndex, MazePoint> = HashMap::new();
        // フェーズ1: すべてのPathをノードとして追加
        for (point, status) in maze_points {
            if status.is_path() {
                Self::add_path_node(&mut graph, &mut point_to_node, &mut node_to_point, *point);
            }
        }

        // フェーズ2: 隣接するPath同士をエッジで接続
        for (point, status) in maze_points {
            if status.is_path() {
                Self::connect_adjacent_paths(
                    &mut graph,
                    &mut point_to_node,
                    x_size,
                    y_size,
                    point,
                    maze_points,
                )?;
            }
        }

        Ok(Self {
            graph,
            point_to_node,
            node_to_point,
        })
    }

    /// Pathノードをグラフに追加する
    ///
    /// # Arguments
    ///
    /// * `point` - 追加するPath座標
    fn add_path_node(
        graph: &mut UnGraph<MazePoint, ()>,
        point_to_node: &mut HashMap<MazePoint, NodeIndex>,
        node_to_point: &mut HashMap<NodeIndex, MazePoint>,
        point: MazePoint,
    ) {
        if let std::collections::hash_map::Entry::Vacant(e) = point_to_node.entry(point) {
            let node_index = graph.add_node(point);
            e.insert(node_index);
            node_to_point.insert(node_index, point);
        }
    }

    /// 指定座標の隣接Pathとの接続を確立する
    ///
    /// # Arguments
    ///
    /// * `point` - 基準となるPath座標
    /// * `all_points` - 迷路の全座標とその状態
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(())、失敗した場合はエラー
    fn connect_adjacent_paths(
        graph: &mut UnGraph<MazePoint, ()>,
        point_to_node: &mut HashMap<MazePoint, NodeIndex>,
        x_size: u64,
        y_size: u64,
        point: &MazePoint,
        all_points: &HashMap<MazePoint, MazePointStatus>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_node = point_to_node.get(point).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Node not found for point {:?}", point),
            )) as Box<dyn std::error::Error>
        })?;

        // 4方向の隣接座標をチェック
        let adjacent_offsets = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in adjacent_offsets {
            let new_x = point.x() as i64 + dx as i64;
            let new_y = point.y() as i64 + dy as i64;

            // 境界チェック
            if new_x >= 0 && new_y >= 0 && new_x < x_size as i64 && new_y < y_size as i64 {
                let adjacent_point = MazePoint::new(new_x as u64, new_y as u64);

                // 隣接点がPathかチェック
                if let Some(adjacent_status) = all_points.get(&adjacent_point)
                    && adjacent_status.is_path()
                {
                    // 隣接PathのNodeIndexを取得
                    if let Some(&adjacent_node) = point_to_node.get(&adjacent_point) {
                        // エッジが存在しない場合のみ追加
                        if !graph.contains_edge(*current_node, adjacent_node) {
                            graph.add_edge(*current_node, adjacent_node, ());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::maze_cell::maze_point::point::MazePoint;
    use crate::maze::maze_cell::maze_point::point_status::MazePointStatus;
    use std::collections::HashMap;

    #[test]
    fn test_build_from_maze_points_simple_path() {
        // 2x2 の迷路: (0,0)と(1,0)が通路、他は壁
        let mut maze_points = HashMap::new();
        use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
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

        let graph = PathConnectivityGraph::build_from_maze_points(&maze_points).unwrap();
        // ノード数は2
        assert_eq!(graph.graph.node_count(), 2);
        // エッジ数は1（(0,0)-(1,0)）
        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_build_from_maze_points_disconnected() {
        // 2x2 の迷路: (0,0)と(1,1)が通路、他は壁
        let mut maze_points = HashMap::new();
        use crate::maze::maze_cell::wall::wall_identifier::WallIdentifier;
        maze_points.insert(
            MazePoint::new(0, 0),
            MazePointStatus::new_not_resolved_path(),
        );
        maze_points.insert(
            MazePoint::new(1, 1),
            MazePointStatus::new_not_resolved_path(),
        );
        maze_points.insert(
            MazePoint::new(1, 0),
            MazePointStatus::new_maze_wall(WallIdentifier::new()),
        );
        maze_points.insert(
            MazePoint::new(0, 1),
            MazePointStatus::new_maze_wall(WallIdentifier::new()),
        );

        let graph = PathConnectivityGraph::build_from_maze_points(&maze_points).unwrap();
        // ノード数は2
        assert_eq!(graph.graph.node_count(), 2);
        // エッジ数は0（つながっていない）
        assert_eq!(graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_build_from_maze_points_full_path() {
        // 2x2 の全てが通路
        let mut maze_points = HashMap::new();
        for x in 0..2 {
            for y in 0..2 {
                maze_points.insert(
                    MazePoint::new(x, y),
                    MazePointStatus::new_not_resolved_path(),
                );
            }
        }
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_points).unwrap();
        // ノード数は4
        assert_eq!(graph.graph.node_count(), 4);
        // エッジ数は4（上下左右で4本）
        assert_eq!(graph.graph.edge_count(), 4);
    }
}
