use crate::{maze_field::MazePoints, maze_point::MazePoint, maze_point_status::MazePointStatus};
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

/// Pathの連結性を分析するためのグラフ構造
#[derive(Debug)]
pub struct PathConnectivityGraph {
    /// petgraphのグラフインスタンス
    pub(crate) graph: UnGraph<MazePoint, ()>,
    /// MazePointからNodeIndexへのマッピング
    pub(crate) point_to_node: HashMap<MazePoint, NodeIndex>,
    /// NodeIndexからMazePointへのマッピング
    pub(crate) node_to_point: HashMap<NodeIndex, MazePoint>,
    /// 迷路のサイズ情報
    pub(crate) x_size: u32,
    pub(crate) y_size: u32,
}

impl PathConnectivityGraph {
    #[cfg(test)]
    pub fn new_empty(x_size: u32, y_size: u32) -> Self {
        Self::new(x_size, y_size)
    }

    /// 新しいPathConnectivityGraphを作成する
    ///
    /// # Arguments
    ///
    /// * `x_size` - 迷路のX方向サイズ
    /// * `y_size` - 迷路のY方向サイズ
    ///
    /// # Returns
    ///
    /// 新しいPathConnectivityGraphインスタンス
    fn new(x_size: u32, y_size: u32) -> Self {
        Self {
            graph: UnGraph::new_undirected(),
            point_to_node: HashMap::new(),
            node_to_point: HashMap::new(),
            x_size,
            y_size,
        }
    }

    /// MazePointsからPathの連結性グラフを構築する
    ///
    /// この関数は以下の処理を行います：
    /// 1. すべてのPath状態の座標をノードとして追加
    /// 2. 隣接するPath同士をエッジで接続
    ///
    /// # Arguments
    ///
    /// * `maze_points` - 迷路データ
    ///
    /// # Returns
    ///
    /// 成功した場合は構築されたPathConnectivityGraph、失敗した場合はエラー
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::maze_field::MazePoints;
    /// use crate::post_process::detect_unreachable_path::PathConnectivityGraph;
    ///
    /// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
    /// let maze_guard = maze_points.read().unwrap();
    /// let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;
    /// ```
    pub fn build_from_maze_points(
        maze_points: &MazePoints,
    ) -> Result<PathConnectivityGraph, Box<dyn std::error::Error>> {
        let mut me: PathConnectivityGraph =
            PathConnectivityGraph::new(maze_points.x_size(), maze_points.y_size());

        let all_points = maze_points.get_all_maze_points_clone();

        // フェーズ1: すべてのPathをノードとして追加
        for (point, status) in &all_points {
            if matches!(status, MazePointStatus::Path) {
                me.add_path_node(*point);
            }
        }

        // フェーズ2: 隣接するPath同士をエッジで接続
        for (point, status) in &all_points {
            if matches!(status, MazePointStatus::Path) {
                me.connect_adjacent_paths(point, &all_points)?;
            }
        }

        Ok(me)
    }

    /// Pathノードをグラフに追加する
    ///
    /// # Arguments
    ///
    /// * `point` - 追加するPath座標
    fn add_path_node(&mut self, point: MazePoint) {
        if !self.point_to_node.contains_key(&point) {
            let node_index = self.graph.add_node(point);
            self.point_to_node.insert(point, node_index);
            self.node_to_point.insert(node_index, point);
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
        &mut self,
        point: &MazePoint,
        all_points: &HashMap<MazePoint, MazePointStatus>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_node = self.point_to_node.get(point).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Node not found for point {:?}", point),
            )) as Box<dyn std::error::Error>
        })?;

        // 4方向の隣接座標をチェック
        let adjacent_offsets = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in adjacent_offsets {
            let new_x = point.x() as i32 + dx;
            let new_y = point.y() as i32 + dy;

            // 境界チェック
            if new_x >= 0 && new_y >= 0 && new_x < self.x_size as i32 && new_y < self.y_size as i32
            {
                let adjacent_point = MazePoint::new(new_x as u32, new_y as u32);

                // 隣接点がPathかチェック
                if let Some(adjacent_status) = all_points.get(&adjacent_point)
                    && matches!(adjacent_status, MazePointStatus::Path)
                {
                    // 隣接PathのNodeIndexを取得
                    if let Some(&adjacent_node) = self.point_to_node.get(&adjacent_point) {
                        // エッジが存在しない場合のみ追加
                        if !self.graph.contains_edge(*current_node, adjacent_node) {
                            self.graph.add_edge(*current_node, adjacent_node, ());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
