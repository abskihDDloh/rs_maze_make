use crate::maze_field::MazePoints;
use crate::maze_point::MazePoint;
use crate::post_process::path_connectivity_graph::PathConnectivityGraph;
use log::{debug, info};
use petgraph::algo::{simple_paths::all_simple_paths, tarjan_scc};
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;
use std::sync::{Arc, RwLock};

/// ループ検出結果を表す構造体
#[derive(Debug, Clone, Default)]
pub struct LoopDetectionResult {
    /// 検出されたループのリスト（各ループは経路上の点のセット）
    loops: Vec<HashSet<MazePoint>>,
    /// ループに含まれる点の総数
    loop_points_count: usize,
    /// 最大ループのサイズ
    max_loop_size: usize,
    /// ループの総数
    loop_count: usize,
}

impl LoopDetectionResult {
    /// 新しいループ検出結果を作成
    pub fn new(loops: Vec<HashSet<MazePoint>>) -> Self {
        let loop_count = loops.len();
        let loop_points_count = loops.iter().map(|l| l.len()).sum();
        let max_loop_size = loops.iter().map(|l| l.len()).max().unwrap_or(0);

        Self {
            loops,
            loop_points_count,
            max_loop_size,
            loop_count,
        }
    }

    /// 検出されたループのリストを取得
    pub fn loops(&self) -> &Vec<HashSet<MazePoint>> {
        &self.loops
    }

    /// ループに含まれる点の総数を取得
    pub fn loop_points_count(&self) -> usize {
        self.loop_points_count
    }

    /// 最大ループのサイズを取得
    pub fn max_loop_size(&self) -> usize {
        self.max_loop_size
    }

    /// ループの総数を取得
    pub fn loop_count(&self) -> usize {
        self.loop_count
    }

    /// ループが検出されたかどうか
    pub fn has_loops(&self) -> bool {
        !self.loops.is_empty()
    }
}

/// 分岐・合流ペアを表す構造体
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchMergePair {
    /// 分岐点（始点）
    branch_point: MazePoint,
    /// 合流点（終点）
    merge_point: MazePoint,
    /// このペア間の経路数
    path_count: usize,
    /// 推定距離
    distance: usize,
}

impl BranchMergePair {
    pub fn new(branch_point: MazePoint, merge_point: MazePoint, path_count: usize) -> Self {
        let distance = Self::calculate_manhattan_distance(&branch_point, &merge_point);
        Self {
            branch_point,
            merge_point,
            path_count,
            distance,
        }
    }

    pub fn branch_point(&self) -> MazePoint {
        self.branch_point
    }

    pub fn merge_point(&self) -> MazePoint {
        self.merge_point
    }

    pub fn path_count(&self) -> usize {
        self.path_count
    }

    pub fn distance(&self) -> usize {
        self.distance
    }

    fn calculate_manhattan_distance(from: &MazePoint, to: &MazePoint) -> usize {
        let dx = if from.x() > to.x() {
            from.x() - to.x()
        } else {
            to.x() - from.x()
        };
        let dy = if from.y() > to.y() {
            from.y() - to.y()
        } else {
            to.y() - from.y()
        };
        (dx + dy) as usize
    }
}

/// 分岐・合流ペアの検出結果
#[derive(Debug, Clone)]
pub struct BranchMergePairResult {
    /// 検出されたペアのリスト
    pairs: Vec<BranchMergePair>,
    /// 総ペア数
    total_pairs: usize,
}

impl BranchMergePairResult {
    pub fn new(pairs: Vec<BranchMergePair>) -> Self {
        let total_pairs = pairs.len();
        Self { pairs, total_pairs }
    }

    pub fn pairs(&self) -> &Vec<BranchMergePair> {
        &self.pairs
    }

    pub fn total_pairs(&self) -> usize {
        self.total_pairs
    }
}

/// 分岐・合流経路の要点を表す構造体
#[derive(Debug, Clone, PartialEq)]
pub struct BranchMergeKeyPoints {
    /// 分岐点
    branch_point: MazePoint,
    /// 分岐点の次の点
    branch_next_point: MazePoint,
    /// 合流点の一つ前の点
    merge_prev_point: MazePoint,
    /// 合流点
    merge_point: MazePoint,
    /// 経路のID
    path_id: usize,
}

impl BranchMergeKeyPoints {
    pub fn new(
        branch_point: MazePoint,
        branch_next_point: MazePoint,
        merge_prev_point: MazePoint,
        merge_point: MazePoint,
        path_id: usize,
    ) -> Self {
        Self {
            branch_point,
            branch_next_point,
            merge_prev_point,
            merge_point,
            path_id,
        }
    }

    pub fn branch_point(&self) -> MazePoint {
        self.branch_point
    }

    pub fn branch_next_point(&self) -> MazePoint {
        self.branch_next_point
    }

    pub fn merge_prev_point(&self) -> MazePoint {
        self.merge_prev_point
    }

    pub fn merge_point(&self) -> MazePoint {
        self.merge_point
    }

    pub fn path_id(&self) -> usize {
        self.path_id
    }

    /// 要点のリストを取得
    pub fn get_key_points(&self) -> Vec<MazePoint> {
        vec![
            self.branch_point,
            self.branch_next_point,
            self.merge_prev_point,
            self.merge_point,
        ]
    }

    /// フォーマットされた文字列を取得
    pub fn format(&self) -> String {
        format!(
            "経路 {}: 分岐点 {:?} → 次の点 {:?} → 前の点 {:?} → 合流点 {:?}",
            self.path_id,
            self.branch_point,
            self.branch_next_point,
            self.merge_prev_point,
            self.merge_point
        )
    }
}

/// 分岐・合流ペアの要点群を表す構造体
#[derive(Debug, Clone)]
pub struct BranchMergeKeyPointsPair {
    /// 分岐点
    branch_point: MazePoint,
    /// 合流点
    merge_point: MazePoint,
    /// 各経路の要点リスト
    path_key_points: Vec<BranchMergeKeyPoints>,
    /// 経路数
    path_count: usize,
}

impl BranchMergeKeyPointsPair {
    pub fn new(
        branch_point: MazePoint,
        merge_point: MazePoint,
        path_key_points: Vec<BranchMergeKeyPoints>,
    ) -> Self {
        let path_count = path_key_points.len();
        Self {
            branch_point,
            merge_point,
            path_key_points,
            path_count,
        }
    }

    pub fn branch_point(&self) -> MazePoint {
        self.branch_point
    }

    pub fn merge_point(&self) -> MazePoint {
        self.merge_point
    }

    pub fn path_key_points(&self) -> &Vec<BranchMergeKeyPoints> {
        &self.path_key_points
    }

    pub fn path_count(&self) -> usize {
        self.path_count
    }

    /// フォーマットされた詳細情報
    pub fn format_details(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "=== 分岐・合流ペア: {:?} → {:?} ===\n",
            self.branch_point, self.merge_point
        ));
        output.push_str(&format!("経路数: {}\n\n", self.path_count));

        for key_points in &self.path_key_points {
            output.push_str(&format!("{}\n", key_points.format()));
        }

        output
    }
}

/// 分岐・合流要点検出結果
#[derive(Debug, Clone)]
pub struct BranchMergeKeyPointsResult {
    /// 検出されたペアの要点群
    pairs: Vec<BranchMergeKeyPointsPair>,
    /// 総ペア数
    total_pairs: usize,
    /// 総経路数
    total_paths: usize,
}

impl BranchMergeKeyPointsResult {
    pub fn new(pairs: Vec<BranchMergeKeyPointsPair>) -> Self {
        let total_pairs = pairs.len();
        let total_paths = pairs.iter().map(|p| p.path_count()).sum();

        Self {
            pairs,
            total_pairs,
            total_paths,
        }
    }

    pub fn pairs(&self) -> &Vec<BranchMergeKeyPointsPair> {
        &self.pairs
    }

    pub fn total_pairs(&self) -> usize {
        self.total_pairs
    }

    pub fn total_paths(&self) -> usize {
        self.total_paths
    }

    /// 結果のサマリーをフォーマット
    pub fn format_summary(&self) -> String {
        let mut output = String::new();

        output.push_str("=== 分岐・合流要点検出結果 ===\n");
        output.push_str(&format!("検出ペア数: {}\n", self.total_pairs));
        output.push_str(&format!("総経路数: {}\n", self.total_paths));

        if self.total_pairs > 0 {
            let avg_paths = self.total_paths as f64 / self.total_pairs as f64;
            output.push_str(&format!("ペア当たり平均経路数: {:.1}\n", avg_paths));
        }

        output.push_str("\n");
        output
    }
}

/// グラフ内の分岐点を検出する
fn find_branch_points(
    graph: &PathConnectivityGraph,
) -> Result<Vec<petgraph::graph::NodeIndex>, Box<dyn std::error::Error>> {
    let mut branch_points = Vec::new();

    for node_index in graph.graph.node_indices() {
        let neighbor_count = graph.graph.neighbors(node_index).count();

        // 3つ以上の接続を持つノードを分岐点とする
        if neighbor_count >= 3 {
            branch_points.push(node_index);

            if let Some(&point) = graph.node_to_point.get(&node_index) {
                debug!("分岐点検出: {:?} ({} 接続)", point, neighbor_count);
            }
        }
    }

    Ok(branch_points)
}

/// グラフ内の合流点を検出する
fn find_merge_points(
    graph: &PathConnectivityGraph,
) -> Result<Vec<petgraph::graph::NodeIndex>, Box<dyn std::error::Error>> {
    let mut merge_points = Vec::new();

    for node_index in graph.graph.node_indices() {
        let neighbor_count = graph.graph.neighbors(node_index).count();

        // 2つ以上の接続を持つノードを合流点の候補とする
        if neighbor_count >= 2 {
            merge_points.push(node_index);
        }
    }

    Ok(merge_points)
}

/// 2つのノード間の異なる経路数をカウントする
fn count_distinct_paths(
    graph: &PathConnectivityGraph,
    start_node: petgraph::graph::NodeIndex,
    end_node: petgraph::graph::NodeIndex,
) -> Result<usize, Box<dyn std::error::Error>> {
    let max_path_length = 20;

    let paths_iter = all_simple_paths::<Vec<_>, _, RandomState>(
        &graph.graph,
        start_node,
        end_node,
        1,
        Some(max_path_length),
    );

    let paths: Vec<Vec<petgraph::graph::NodeIndex>> = paths_iter.collect();

    Ok(paths.len())
}

/// PathConnectivityGraphから分岐・合流ペアを検出する関数
pub fn detect_branch_merge_pairs(
    graph: &PathConnectivityGraph,
) -> Result<BranchMergePairResult, Box<dyn std::error::Error>> {
    info!("分岐・合流ペア検出を開始");

    let mut pairs = Vec::new();
    let mut processed_pairs = HashSet::new();

    // 1. 分岐点を特定（3つ以上の接続を持つノード）
    let branch_points = find_branch_points(graph)?;
    debug!("検出された分岐点数: {}", branch_points.len());

    // 2. 合流点を特定（複数の経路から到達可能なノード）
    let merge_points = find_merge_points(graph)?;
    debug!("検出された合流点数: {}", merge_points.len());

    // 3. 各分岐点から各合流点への複数経路の存在をチェック
    for &branch_node in &branch_points {
        for &merge_node in &merge_points {
            if branch_node == merge_node {
                continue; // 同じノードはスキップ
            }

            // 分岐点から合流点への複数経路を検出
            let path_count = count_distinct_paths(graph, branch_node, merge_node)?;

            if path_count >= 2
                && let (Some(&branch_point), Some(&merge_point)) = (
                    graph.node_to_point.get(&branch_node),
                    graph.node_to_point.get(&merge_node),
                ) {
                    let pair = BranchMergePair::new(branch_point, merge_point, path_count);

                    // 重複チェック
                    if !processed_pairs.contains(&(branch_point, merge_point)) {
                        debug!(
                            "分岐・合流ペア検出: {:?} → {:?} ({} 経路)",
                            branch_point, merge_point, path_count
                        );
                        pairs.push(pair);
                        processed_pairs.insert((branch_point, merge_point));
                    }
                }
        }
    }

    // 距離順にソート（近い順）
    pairs.sort_by_key(|pair| pair.distance());

    let result = BranchMergePairResult::new(pairs);

    info!("分岐・合流ペア検出完了: {} ペア検出", result.total_pairs());

    Ok(result)
}

/// 指定されたMazePointに対応するノードインデックスを検索
fn find_node_by_point(
    graph: &PathConnectivityGraph,
    point: &MazePoint,
) -> Result<Option<petgraph::graph::NodeIndex>, Box<dyn std::error::Error>> {
    for (&node_index, &node_point) in &graph.node_to_point {
        if node_point == *point {
            return Ok(Some(node_index));
        }
    }
    Ok(None)
}

/// PathConnectivityGraphから分岐・合流の要点を抽出する関数
pub fn extract_branch_merge_key_points(
    graph: &PathConnectivityGraph,
) -> Result<BranchMergeKeyPointsResult, Box<dyn std::error::Error>> {
    info!("分岐・合流要点抽出を開始");

    let mut result_pairs = Vec::new();

    // 1. 分岐・合流ペアを検出
    let pair_result = detect_branch_merge_pairs(graph)?;

    info!("検出された分岐・合流ペア数: {}", pair_result.total_pairs());

    // 2. 各ペアについて要点を抽出
    for pair in pair_result.pairs() {
        let branch_node = find_node_by_point(graph, &pair.branch_point())?;
        let merge_node = find_node_by_point(graph, &pair.merge_point())?;

        if let (Some(b_node), Some(m_node)) = (branch_node, merge_node) {
            debug!(
                "要点抽出中: {:?} → {:?}",
                pair.branch_point(),
                pair.merge_point()
            );

            // 全経路を取得
            let paths = find_all_simple_paths_between_nodes(graph, b_node, m_node)?;

            if !paths.is_empty() {
                let key_points_list =
                    extract_key_points_from_paths(&paths, pair.branch_point(), pair.merge_point())?;

                if !key_points_list.is_empty() {
                    let pair_key_points = BranchMergeKeyPointsPair::new(
                        pair.branch_point(),
                        pair.merge_point(),
                        key_points_list,
                    );

                    info!(
                        "要点抽出完了: {:?} → {:?} ({} 経路)",
                        pair.branch_point(),
                        pair.merge_point(),
                        pair_key_points.path_count()
                    );

                    result_pairs.push(pair_key_points);
                }
            }
        }
    }

    let result = BranchMergeKeyPointsResult::new(result_pairs);

    info!(
        "分岐・合流要点抽出完了: {} ペア, {} 総経路",
        result.total_pairs(),
        result.total_paths()
    );

    Ok(result)
}

/// 経路から要点を抽出する
fn extract_key_points_from_paths(
    paths: &[Vec<MazePoint>],
    expected_branch: MazePoint,
    expected_merge: MazePoint,
) -> Result<Vec<BranchMergeKeyPoints>, Box<dyn std::error::Error>> {
    let mut key_points_list = Vec::new();

    for (path_id, path) in paths.iter().enumerate() {
        if path.len() < 2 {
            debug!("経路 {} は短すぎます（長さ: {}）", path_id, path.len());
            continue;
        }

        // 要点を抽出
        let branch_point = path[0];
        let merge_point = path[path.len() - 1];

        // 期待される分岐点・合流点と一致するかチェック
        if branch_point != expected_branch || merge_point != expected_merge {
            debug!(
                "経路 {} の端点が期待値と異なります: 実際({:?} → {:?}) vs 期待({:?} → {:?})",
                path_id, branch_point, merge_point, expected_branch, expected_merge
            );
            continue;
        }

        // 分岐点の次の点
        let branch_next_point = if path.len() >= 2 {
            path[1]
        } else {
            debug!("経路 {} には分岐点の次の点がありません", path_id);
            continue;
        };

        // 合流点の一つ前の点
        let merge_prev_point = if path.len() >= 3 {
            path[path.len() - 2]
        } else {
            // 長さが2の場合は分岐点の次の点と合流点の前の点が同じ
            branch_next_point
        };

        let key_points = BranchMergeKeyPoints::new(
            branch_point,
            branch_next_point,
            merge_prev_point,
            merge_point,
            path_id,
        );

        debug!(
            "経路 {} の要点: {:?} → {:?} → {:?} → {:?}",
            path_id, branch_point, branch_next_point, merge_prev_point, merge_point
        );

        key_points_list.push(key_points);
    }

    Ok(key_points_list)
}

/// 2つのノード間の全ての単純経路を検索（修正版）
fn find_all_simple_paths_between_nodes(
    graph: &PathConnectivityGraph,
    start_node: petgraph::graph::NodeIndex,
    end_node: petgraph::graph::NodeIndex,
) -> Result<Vec<Vec<MazePoint>>, Box<dyn std::error::Error>> {
    let max_path_length = 25;

    // 型推論を活用した経路検索
    let paths_iter = all_simple_paths::<Vec<_>, _, RandomState>(
        &graph.graph,
        start_node,
        end_node,
        0,
        Some(max_path_length),
    );

    let node_paths: Vec<Vec<petgraph::graph::NodeIndex>> = paths_iter.collect();

    // ノードをMazePointに変換
    let mut maze_point_paths = Vec::new();

    for node_path in node_paths {
        let mut maze_point_path = Vec::new();

        for node_index in node_path {
            if let Some(&point) = graph.node_to_point.get(&node_index) {
                maze_point_path.push(point);
            } else {
                maze_point_path.clear();
                break;
            }
        }

        if !maze_point_path.is_empty() {
            maze_point_paths.push(maze_point_path);
        }
    }

    debug!(
        "ノード {:?} → {:?} 間で {} 経路を検出",
        start_node,
        end_node,
        maze_point_paths.len()
    );

    Ok(maze_point_paths)
}

/// 便利な公開関数：迷路から直接要点を抽出
pub fn extract_branch_merge_key_points_from_maze(
    maze_points: &Arc<RwLock<MazePoints>>,
) -> Result<BranchMergeKeyPointsResult, Box<dyn std::error::Error>> {
    info!("迷路からの分岐・合流要点抽出を開始");

    let maze_guard = maze_points
        .read()
        .map_err(|_| Box::new(std::io::Error::other("Failed to acquire read lock")))?;

    // PathConnectivityGraphを構築
    let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

    debug!(
        "要点抽出用グラフ構築: {} ノード, {} エッジ",
        graph.graph.node_count(),
        graph.graph.edge_count()
    );

    // グラフから要点を抽出
    let result = extract_branch_merge_key_points(&graph)?;

    debug!(
        "迷路からの要点抽出完了: {} ペア, {} 総経路",
        result.total_pairs(),
        result.total_paths()
    );

    Ok(result)
}

/// 特定の分岐・合流ペアの要点のみを抽出
pub fn extract_key_points_for_specific_pair(
    graph: &PathConnectivityGraph,
    branch_point: MazePoint,
    merge_point: MazePoint,
) -> Result<Option<BranchMergeKeyPointsPair>, Box<dyn std::error::Error>> {
    let branch_node = find_node_by_point(graph, &branch_point)?;
    let merge_node = find_node_by_point(graph, &merge_point)?;

    if let (Some(b_node), Some(m_node)) = (branch_node, merge_node) {
        let paths = find_all_simple_paths_between_nodes(graph, b_node, m_node)?;

        if !paths.is_empty() {
            let key_points_list = extract_key_points_from_paths(&paths, branch_point, merge_point)?;

            if !key_points_list.is_empty() {
                let pair_key_points =
                    BranchMergeKeyPointsPair::new(branch_point, merge_point, key_points_list);

                return Ok(Some(pair_key_points));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_branch_merge_key_points() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(15, 15)?;
        let result = extract_branch_merge_key_points_from_maze(&maze_points)?;

        info!("分岐・合流要点抽出テスト結果:");
        println!("{}", result.format_summary());

        info!("検出されたペア数: {}", result.total_pairs());
        info!("総経路数: {}", result.total_paths());

        // 各ペアの要点詳細をログ出力
        for (i, pair) in result.pairs().iter().enumerate() {
            info!("ペア {}: {}", i + 1, pair.format_details());

            for key_points in pair.path_key_points() {
                info!("  {}", key_points.format());

                let points = key_points.get_key_points();
                info!("    要点リスト: {:?}", points);
            }
        }

        Ok(())
    }

    #[test]
    fn test_key_points_structure() {
        let branch_point = MazePoint::new(1, 1);
        let branch_next = MazePoint::new(1, 3);
        let merge_prev = MazePoint::new(3, 3);
        let merge_point = MazePoint::new(5, 3);

        let key_points =
            BranchMergeKeyPoints::new(branch_point, branch_next, merge_prev, merge_point, 0);

        assert_eq!(key_points.branch_point(), branch_point);
        assert_eq!(key_points.branch_next_point(), branch_next);
        assert_eq!(key_points.merge_prev_point(), merge_prev);
        assert_eq!(key_points.merge_point(), merge_point);
        assert_eq!(key_points.path_id(), 0);

        let points = key_points.get_key_points();
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], branch_point);
        assert_eq!(points[1], branch_next);
        assert_eq!(points[2], merge_prev);
        assert_eq!(points[3], merge_point);

        info!("作成された要点: {}", key_points.format());
    }

    #[test]
    fn test_specific_pair_key_points() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(13, 13)?;

        let maze_guard = maze_points
            .read()
            .map_err(|_| Box::new(std::io::Error::other("Failed to acquire read lock")))?;

        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

        // 仮の分岐・合流点でテスト
        let branch_point = MazePoint::new(1, 1);
        let merge_point = MazePoint::new(5, 5);

        let result = extract_key_points_for_specific_pair(&graph, branch_point, merge_point)?;

        match result {
            Some(pair_key_points) => {
                info!("特定ペアの要点抽出成功:");
                info!("{}", pair_key_points.format_details());
            }
            None => {
                info!("指定された分岐・合流ペアの要点は見つかりませんでした");
            }
        }

        Ok(())
    }
}
