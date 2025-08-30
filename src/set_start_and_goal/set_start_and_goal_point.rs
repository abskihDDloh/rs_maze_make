//! # set_start_and_goal_point モジュール
//!
//! 迷路のスタート・ゴール座標を自動的に決定し、両方を返すユーティリティを提供します。
//!
//! ## 概要
//! - 迷路の全座標点（`maze_points`）から、
//!   1. 通路グラフ上でランダムな点から最遠の点を「スタート」としてマーク
//!   2. そのスタート点からさらに最遠の点を「ゴール」としてマーク
//! - スタート・ゴールは`MazePointStatus::StartOrEnd`でマークされます。
//!
//! ## 例
//! ```rust
//! let mut maze_points = ...; // 迷路データを用意
//! let (start, goal) = set_start_and_goal_point(&mut maze_points)?;
//! println!("start={:?}, goal={:?}", start, goal);
//! ```
//!
//! ## 関連関数
//! - [`mark_farthest_point_as_start_or_end`] : スタート決定
//! - [`set_start_and_goal_point`] : スタート・ゴール決定
//!
use std::collections::HashMap;

use rand::seq::IteratorRandom;

use crate::{
    maze::maze_cell::maze_point::{point::MazePoint, point_status::MazePointStatus},
    set_start_and_goal::{
        path_connectivity_graph::PathConnectivityGraph, start_and_goal_point::StartAndGoalPoint,
    },
};

/// 迷路のスタート・ゴール座標を決定するための構造体。
pub struct StartAndGoalSetter {
    /// 迷路の全座標点とその状態
    all_maze_points: HashMap<MazePoint, MazePointStatus>,
    /// 通路の接続グラフ
    graph: PathConnectivityGraph,
    /// スタート・ゴール座標
    start_and_goal: StartAndGoalPoint,
}

impl StartAndGoalSetter {
    /// 新しいStartAndGoalSetterを生成します。
    ///
    /// # 引数
    /// * `all_maze_points` - 迷路の全座標点と状態
    pub fn new(all_maze_points: HashMap<MazePoint, MazePointStatus>) -> Self {
        let graph = PathConnectivityGraph::build_from_maze_points(&all_maze_points);
        // graphの生成に失敗した場合は空のグラフを使用
        StartAndGoalSetter {
            all_maze_points: all_maze_points.clone(),
            graph: graph.unwrap_or_default(),
            start_and_goal: StartAndGoalPoint::default(),
        }
    }

    /// 迷路の全座標点とその状態のクローンを返します。
    pub fn get_all_maze_points_clone(&self) -> HashMap<MazePoint, MazePointStatus> {
        self.all_maze_points.clone()
    }

    /// スタート・ゴール座標のクローンを返します。
    #[allow(dead_code)]
    pub fn get_start_and_goal_clone(&self) -> StartAndGoalPoint {
        self.start_and_goal.clone()
    }

    /// 通路グラフ上でランダムな点から最遠の点を「スタート」または「ゴール」としてマークします。
    fn mark_farthest_point_as_start_or_end(
        &mut self,
    ) -> Result<MazePoint, Box<dyn std::error::Error>> {
        if self.graph.graph.node_count() == 0 {
            return Err("No path node found".into());
        }
        // 1. ランダムなノードを選択
        let mut rng = rand::rng();

        let start_node = self
            .graph
            .graph
            .node_indices()
            .choose(&mut rng)
            .ok_or("Failed to select random node")?;

        // 2. 幅優先探索で最遠点を求める
        use petgraph::visit::Bfs;
        let mut bfs = Bfs::new(&self.graph.graph, start_node);
        let mut last_node = start_node;
        while let Some(node) = bfs.next(&self.graph.graph) {
            last_node = node;
        }
        // 3. 最遠点のMazePointを取得
        let farthest_point = self.graph.node_to_point.get(&last_node).ok_or(format!(
            "NodeIndex to MazePoint mapping failed. {:?}",
            last_node
        ))?;
        // 4. maze_pointsにStartOrEndとしてマーク
        self.all_maze_points
            .insert(*farthest_point, MazePointStatus::new_start_or_end_path());
        Ok(*farthest_point)
    }

    /// スタート・ゴール座標を決定し、構造体に格納して返します。
    ///
    /// # 戻り値
    /// * `Ok(StartAndGoalPoint)` - スタート・ゴール座標
    /// * `Err(_)` - 決定に失敗した場合
    pub fn set_start_and_goal_point(
        &mut self,
    ) -> Result<StartAndGoalPoint, Box<dyn std::error::Error>> {
        let farthest_point: MazePoint = self.mark_farthest_point_as_start_or_end()?;
        let start_point = farthest_point;
        // 1. farthest_pointがall_maze_pointsに含まれているか確認
        let status = self
            .all_maze_points
            .get(&farthest_point)
            .ok_or("farthest_point not found in all_maze_points")?;
        // 2. pathかつStartOrEndであることを確認
        if !status.is_path() || !status.is_start_or_end_path() {
            return Err("farthest_point is not a StartOrEnd path".into());
        }
        // 3. farthest_pointに対応するノードを取得
        let &start_node = self
            .graph
            .point_to_node
            .get(&farthest_point)
            .ok_or("farthest_point not found in graph nodes")?;
        // 4. 幅優先探索で最遠点を求める
        use petgraph::visit::Bfs;
        let mut bfs = Bfs::new(&self.graph.graph, start_node);
        let mut last_node = start_node;
        while let Some(node) = bfs.next(&self.graph.graph) {
            last_node = node;
        }
        // 5. 最遠点のMazePointを取得
        let goal_point_ref = self
            .graph
            .node_to_point
            .get(&last_node)
            .ok_or("NodeIndex to MazePoint mapping failed")?;
        let goal_point = *goal_point_ref;
        // 6. all_maze_pointsにStartOrEndとしてマーク
        self.all_maze_points
            .insert(goal_point, MazePointStatus::new_start_or_end_path());
        let sg = StartAndGoalPoint {
            start: start_point,
            goal: goal_point,
        };
        self.start_and_goal = sg.clone();
        Ok(sg.clone())
    }
}
