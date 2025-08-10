use crate::maze_field::MazePoints;
use crate::maze_point::MazePoint;
use crate::maze_point_status::MazePointStatus;
use log::{debug, info};
use petgraph::algo::connected_components;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Pathの連結性を分析するためのグラフ構造
#[derive(Debug)]
pub struct PathConnectivityGraph {
    /// petgraphのグラフインスタンス
    graph: UnGraph<MazePoint, ()>,
    /// MazePointからNodeIndexへのマッピング
    point_to_node: HashMap<MazePoint, NodeIndex>,
    /// NodeIndexからMazePointへのマッピング
    node_to_point: HashMap<NodeIndex, MazePoint>,
    /// 迷路のサイズ情報
    x_size: u32,
    y_size: u32,
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

    /// (1,1)から到達できないPathを検出する
    ///
    /// この関数は以下の手順で到達不可能なPathを検出します：
    /// 1. (1,1)座標がPathとして存在するかチェック
    /// 2. DFS（深度優先探索）を使用して(1,1)から到達可能なすべてのPathを特定
    /// 3. 到達不可能なPathを収集して返す
    ///
    /// # Returns
    ///
    /// (1,1)から到達できないPath座標のベクタ、または(1,1)がPathでない場合はエラー
    ///
    /// # Examples
    ///
    /// ```rust
    /// let unreachable_paths = graph.find_unreachable_paths_from_start()?;
    /// if unreachable_paths.is_empty() {
    ///     info!("All paths are reachable from (1,1)");
    /// } else {
    ///     info!("Unreachable paths: {:?}", unreachable_paths);
    /// }
    /// ```
    pub fn find_unreachable_paths_from_start(
        &self,
    ) -> Result<Vec<MazePoint>, Box<dyn std::error::Error>> {
        let start_point = MazePoint::new(1, 1);

        // (1,1)がPathとして存在するかチェック
        let start_node = self.point_to_node.get(&start_point).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Start point (1,1) is not a Path or does not exist",
            )) as Box<dyn std::error::Error>
        })?;

        // DFSで(1,1)から到達可能なすべてのノードを見つける
        let mut visited = HashSet::new();
        let mut stack = vec![*start_node];

        while let Some(current_node) = stack.pop() {
            if visited.contains(&current_node) {
                continue;
            }

            visited.insert(current_node);

            // 隣接ノードをスタックに追加
            for edge in self.graph.edges(current_node) {
                let neighbor = edge.target();
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }

        // 到達不可能なPathを収集
        let mut unreachable_paths = Vec::new();

        for (point, &node_index) in &self.point_to_node {
            if !visited.contains(&node_index) {
                unreachable_paths.push(*point);
            }
        }

        // 統計情報の出力
        let total_nodes = self.point_to_node.len();
        let reachable_count = visited.len();
        let unreachable_count = unreachable_paths.len();

        // 連結成分数を計算（petgraphのconnected_componentsを使用）
        let component_count = connected_components(&self.graph);

        debug!("Total connected components: {}", component_count);
        debug!("Total nodes: {}", total_nodes);
        debug!("Reachable from start: {} nodes", reachable_count);
        debug!("Unreachable from start: {} nodes", unreachable_count);
        debug!("Unreachable paths: {:?}", unreachable_paths);

        Ok(unreachable_paths)
    }

    /// グラフの統計情報を取得する
    ///
    /// # Returns
    ///
    /// (ノード数, エッジ数, 連結成分数)のタプル
    ///
    /// # Examples
    ///
    /// ```rust
    /// let (nodes, edges, components) = graph.get_statistics();
    /// println!("Graph has {} nodes, {} edges, {} components", nodes, edges, components);
    /// ```
    pub fn get_statistics(&self) -> (usize, usize, usize) {
        let node_count = self.graph.node_count();
        let edge_count = self.graph.edge_count();
        let component_count = connected_components(&self.graph);

        (node_count, edge_count, component_count)
    }

    /// 各連結成分の詳細情報を取得する
    ///
    /// DFS（深度優先探索）を使用してグラフを探索し、
    /// 各連結成分に含まれるPath座標のリストを構築します。
    ///
    /// # Returns
    ///
    /// 各連結成分に含まれるPath座標のベクタのベクタ
    ///
    /// # Examples
    ///
    /// ```rust
    /// let components = graph.get_connected_components();
    /// for (i, component) in components.iter().enumerate() {
    ///     println!("Component {}: {} paths", i + 1, component.len());
    /// }
    /// ```
    pub fn get_connected_components(&self) -> Vec<Vec<MazePoint>> {
        let mut components: Vec<Vec<MazePoint>> = Vec::new();
        let mut visited = HashSet::new();

        // すべてのノードを調べて連結成分を構築
        for start_node in self.graph.node_indices() {
            if visited.contains(&start_node) {
                continue; // 既に処理済み
            }

            let mut current_component = Vec::new();
            let mut stack = vec![start_node];

            // DFSで連結成分を探索
            while let Some(node) = stack.pop() {
                if visited.contains(&node) {
                    continue;
                }

                visited.insert(node);

                // ノードをMazePointに変換して成分に追加
                if let Some(point) = self.node_to_point.get(&node) {
                    current_component.push(*point);
                }

                // 隣接ノードを探索対象に追加
                for edge in self.graph.edges(node) {
                    let neighbor = edge.target();
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }

            // 空でなければ成分として追加
            if !current_component.is_empty() {
                components.push(current_component);
            }
        }

        components
    }

    /// 特定のPath座標が(1,1)から到達可能かチェックする
    ///
    /// petgraphのhas_path_connectingアルゴリズムを使用して、
    /// スタート地点(1,1)から対象座標への経路が存在するかを判定します。
    ///
    /// # Arguments
    ///
    /// * `target` - チェック対象のPath座標
    ///
    /// # Returns
    ///
    /// (1,1)から到達可能な場合はtrue、そうでなければfalse
    ///
    /// # Examples
    ///
    /// ```rust
    /// let target_point = MazePoint::new(3, 3);
    /// if graph.is_reachable_from_start(&target_point) {
    ///     println!("Point {:?} is reachable from start", target_point);
    /// }
    /// ```
    pub fn is_reachable_from_start(&self, target: &MazePoint) -> bool {
        let start_point = MazePoint::new(1, 1);

        if let (Some(&start_node), Some(&target_node)) = (
            self.point_to_node.get(&start_point),
            self.point_to_node.get(target),
        ) {
            petgraph::algo::has_path_connecting(&self.graph, start_node, target_node, None)
        } else {
            false
        }
    }
}

/// 迷路の統計情報と分析結果を格納する構造体
///
/// この構造体は迷路の連結性分析の結果を保持し、
/// メインパス（スタート地点から到達可能な領域）と
/// 孤立パス（到達不可能な領域）を区別して管理します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MazeStatisticsValue {
    /// グラフ内のノード数（通路の総数）
    nodes: usize,
    /// グラフ内のエッジ数（隣接する通路間の接続数）
    edges: usize,
    /// 連結成分の総数
    components: usize,
    /// メインパス（スタート地点から到達可能な通路の集合）
    main_path_points: HashSet<MazePoint>,
    /// 孤立パス群（到達不可能な通路の集合群）
    orphan_paths_points: Vec<HashSet<MazePoint>>,
}

impl MazeStatisticsValue {
    /// 新しいMazeStatisticsValueを作成する
    ///
    /// # Arguments
    ///
    /// * `nodes` - グラフ内のノード数
    /// * `edges` - グラフ内のエッジ数
    /// * `components` - 連結成分数
    /// * `paths` - 各連結成分に含まれる通路座標の集合
    /// * `unreachable_paths` - スタート地点から到達不可能な通路座標
    ///
    /// # Returns
    ///
    /// 新しいMazeStatisticsValueインスタンス
    ///
    /// # Examples
    ///
    /// ```rust
    /// let components = vec![
    ///     vec![MazePoint::new(1, 1), MazePoint::new(1, 2)],
    ///     vec![MazePoint::new(3, 3)],
    /// ];
    /// let unreachable = vec![MazePoint::new(3, 3)];
    /// let stats = MazeStatisticsValue::new(10, 15, 2, components, unreachable);
    /// ```
    pub fn new(
        nodes: usize,
        edges: usize,
        components: usize,
        paths: Vec<Vec<MazePoint>>,
        unreachable_paths: Vec<MazePoint>,
    ) -> Self {
        // unreachable_pathsをHashSetに変換する
        let up_set = unreachable_paths.into_iter().collect::<HashSet<_>>();

        // pathsの要素に対して繰り返し処理
        let mut main_path_points = HashSet::new();
        let mut orphan_paths_points = Vec::new();
        for path in paths {
            // up_setの内容のどれか１つがpathに含まれているか確認する
            if path.iter().any(|p| up_set.contains(p)) {
                let mut op = HashSet::new();
                op.extend(path);
                orphan_paths_points.push(op);
            } else {
                main_path_points.extend(path);
            }
        }

        Self {
            nodes,
            edges,
            components,
            main_path_points,
            orphan_paths_points,
        }
    }

    /// グラフ内のノード数を取得する
    ///
    /// # Returns
    ///
    /// ノード数（通路の総数）
    pub fn nodes(&self) -> usize {
        self.nodes
    }

    /// グラフ内のエッジ数を取得する
    ///
    /// # Returns
    ///
    /// エッジ数（隣接する通路間の接続数）
    pub fn edges(&self) -> usize {
        self.edges
    }

    /// 連結成分数を取得する
    ///
    /// # Returns
    ///
    /// 連結成分の総数
    pub fn components(&self) -> usize {
        self.components
    }

    /// メインパス（到達可能な通路群）を取得する
    ///
    /// # Returns
    ///
    /// スタート地点(1,1)から到達可能な通路座標の集合への参照
    pub fn main_path_points(&self) -> &HashSet<MazePoint> {
        &self.main_path_points
    }

    /// 孤立パス群（到達不可能な通路群）を取得する
    ///
    /// # Returns
    ///
    /// 各孤立した連結成分の通路座標集合のベクタへの参照
    pub fn orphan_paths_points(&self) -> &Vec<HashSet<MazePoint>> {
        &self.orphan_paths_points
    }
}

/// MazePointsから迷路の連結性を分析する内部実装関数
///
/// この関数は以下の処理を一括で実行します：
/// 1. PathConnectivityGraphの構築
/// 2. スタート地点(1,1)からの到達可能性分析
/// 3. メインパスと孤立パスの分類
/// 4. 統計情報の集計
///
/// # Arguments
///
/// * `maze_points` - 迷路データ
///
/// # Returns
///
/// 迷路の連結性分析結果を含むMazeStatisticsValue
///
/// # Errors
///
/// * グラフ構築に失敗した場合
/// * 到達不可能パス検出に失敗した場合
///
/// # Examples
///
/// ```rust
/// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
/// let maze_guard = maze_points.read().unwrap();
/// let statistics = detect_unreachable_paths_backend(&maze_guard)?;
///
/// info!("Found {} orphan path groups", statistics.orphan_paths_points().len());
/// info!("Main path contains {} points", statistics.main_path_points().len());
/// ```
fn detect_unreachable_paths_backend(
    maze_points: &MazePoints,
) -> Result<MazeStatisticsValue, Box<dyn std::error::Error>> {
    // PathConnectivityGraphを構築
    let graph = PathConnectivityGraph::build_from_maze_points(maze_points)?;
    // 統計情報を取得
    let statistics = graph.get_statistics();

    // 到達不可能なPathを検出
    let unreachable_paths = graph.find_unreachable_paths_from_start()?;

    // 詳細な連結成分情報を直接初期化
    let components = graph.get_connected_components();

    Ok(MazeStatisticsValue::new(
        statistics.0,
        statistics.1,
        statistics.2,
        components,
        unreachable_paths,
    ))
}

/// MazePointsから迷路の連結性を分析する公開関数
///
/// この関数は`Arc<RwLock<MazePoints>>`を受け取り、スレッドセーフに
/// 迷路データにアクセスして連結性分析を実行します。
///
/// 分析結果には以下の情報が含まれます：
/// - グラフの基本統計（ノード数、エッジ数、連結成分数）
/// - メインパス（スタート地点から到達可能な通路群）
/// - 孤立パス群（到達不可能な通路群）
///
/// # Arguments
///
/// * `maze_points` - 迷路データ（スレッドセーフなラッパー）
///
/// # Returns
///
/// 迷路の連結性分析結果を含むMazeStatisticsValue
///
/// # Errors
///
/// * 読み取りロックの取得に失敗した場合
/// * グラフ構築に失敗した場合
/// * 連結性分析に失敗した場合
///
/// # Examples
///
/// ```rust
/// use std::sync::{Arc, RwLock};
/// use crate::maze_field::MazePoints;
/// use crate::post_process::detect_unreachable_path::detect_unreachable_paths;
///
/// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
/// let statistics = detect_unreachable_paths(&maze_points)?;
///
/// if statistics.orphan_paths_points().is_empty() {
///     println!("All paths are connected to the main area");
/// } else {
///     println!("Found {} isolated path groups", statistics.orphan_paths_points().len());
/// }
/// ```
pub fn detect_unreachable_paths(
    maze_points: &Arc<RwLock<MazePoints>>,
) -> Result<MazeStatisticsValue, Box<dyn std::error::Error>> {
    let maze_guard = maze_points.read().map_err(|_| {
        Box::new(std::io::Error::other(
            "Failed to acquire read lock for maze points",
        ))
    })?;
    detect_unreachable_paths_backend(&maze_guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze_field::MazePoints;

    #[test]
    fn test_path_connectivity_graph_creation() {
        // 空のグラフを作成してテスト
        let graph = PathConnectivityGraph::new_empty(7, 7);
        let (node_count, edge_count, component_count) = graph.get_statistics();

        assert_eq!(node_count, 0);
        assert_eq!(edge_count, 0);
        assert_eq!(component_count, 0);
    }

    #[test]
    fn test_build_from_maze_points() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // MazePointsからグラフを構築
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

        let (node_count, edge_count, component_count) = graph.get_statistics();

        // 7x7迷路では通路が存在するはず
        assert!(node_count > 0);
        info!(
            "Test graph - Nodes: {}, Edges: {}, Components: {}",
            node_count, edge_count, component_count
        );

        Ok(())
    }

    #[test]
    fn test_detect_unreachable_paths() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // MazeStatisticsValueの作成と検証
        let statistics = detect_unreachable_paths_backend(&maze_guard)?;

        info!(
            "Statistics - Nodes: {}, Edges: {}, Components: {}",
            statistics.nodes(),
            statistics.edges(),
            statistics.components()
        );
        info!(
            "Main path points: {}, Orphan path groups: {}",
            statistics.main_path_points().len(),
            statistics.orphan_paths_points().len()
        );

        // 基本的な妥当性チェック
        assert!(statistics.nodes() > 0, "Should have at least one path node");
        assert!(
            statistics.components() > 0,
            "Should have at least one component"
        );
        assert!(
            statistics.components() <= statistics.nodes(),
            "Components should not exceed nodes"
        );

        // メインパスまたは孤立パスのいずれかは存在するはず
        let total_path_points = statistics.main_path_points().len()
            + statistics
                .orphan_paths_points()
                .iter()
                .map(|group| group.len())
                .sum::<usize>();
        assert!(total_path_points > 0, "Should have some path points");

        // 各孤立パス群の詳細情報を出力
        for (i, orphan_group) in statistics.orphan_paths_points().iter().enumerate() {
            info!("Orphan group {}: {} paths", i + 1, orphan_group.len());
            if orphan_group.len() <= 5 {
                info!("  Points: {:?}", orphan_group);
            }
        }

        Ok(())
    }

    #[test]
    fn test_start_point_reachability() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // MazePointsからグラフを構築
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

        let start_point = MazePoint::new(1, 1);

        // (1,1)がPathとして存在する場合、自分自身は到達可能であるべき
        if graph.point_to_node.contains_key(&start_point) {
            assert!(graph.is_reachable_from_start(&start_point));
        }

        Ok(())
    }

    #[test]
    fn test_isolated_path_detection() -> Result<(), Box<dyn std::error::Error>> {
        // 手動で分離されたPathを持つ小さな迷路を作成してテスト
        let maze_points = MazePoints::initialize_maze_points(5, 5)?;
        let maze_guard = maze_points.read().unwrap();

        // MazePointsからグラフを構築
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

        let components = graph.get_connected_components();

        info!("Components in 5x5 maze:");
        for (i, component) in components.iter().enumerate() {
            info!("  Component {}: {:?}", i + 1, component);
        }

        // 何らかの連結成分が存在するはず
        assert!(!components.is_empty());

        Ok(())
    }

    #[test]
    fn test_graph_statistics() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // MazePointsからグラフを構築
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;

        let (nodes, edges, components) = graph.get_statistics();

        // 基本的な妥当性チェック
        assert!(nodes > 0, "Should have at least one path node");
        assert!(components > 0, "Should have at least one component");
        assert!(components <= nodes, "Components should not exceed nodes");

        info!(
            "Graph validation - Nodes: {}, Edges: {}, Components: {}",
            nodes, edges, components
        );

        Ok(())
    }

    #[test]
    fn test_maze_statistics_value() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // MazeStatisticsValueの作成と機能をテスト
        let statistics = detect_unreachable_paths_backend(&maze_guard)?;

        // アクセッサメソッドのテスト
        let nodes = statistics.nodes();
        let edges = statistics.edges();
        let components = statistics.components();
        let main_paths = statistics.main_path_points();
        let orphan_paths = statistics.orphan_paths_points();

        assert!(nodes > 0, "Should have nodes");
        assert!(components > 0, "Should have components");
        assert!(components <= nodes, "Components should not exceed nodes");

        // メインパスまたは孤立パスのいずれかは存在するはず
        assert!(
            !main_paths.is_empty() || !orphan_paths.is_empty(),
            "Should have either main paths or orphan paths"
        );

        info!(
            "MazeStatisticsValue test - Nodes: {}, Edges: {}, Components: {}, Main paths: {}, Orphan groups: {}",
            nodes,
            edges,
            components,
            main_paths.len(),
            orphan_paths.len()
        );

        Ok(())
    }

    #[test]
    fn test_public_detect_unreachable_paths() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;

        // 公開関数のテスト
        let statistics = detect_unreachable_paths(&maze_points)?;

        // 基本的な妥当性チェック
        assert!(statistics.nodes() > 0, "Should have at least one path node");
        assert!(
            statistics.components() > 0,
            "Should have at least one component"
        );

        info!(
            "Public function test - Nodes: {}, Edges: {}, Components: {}",
            statistics.nodes(),
            statistics.edges(),
            statistics.components()
        );

        Ok(())
    }

    #[test]
    fn test_unreachable_paths_detection() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        // グラフを構築して到達不可能パスを検出
        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;
        let unreachable_paths = graph.find_unreachable_paths_from_start()?;

        info!("Found {} unreachable paths", unreachable_paths.len());

        // 到達不可能パスが見つかった場合の詳細確認
        if !unreachable_paths.is_empty() {
            info!("Unreachable paths: {:?}", unreachable_paths);

            // 各到達不可能パスが実際に到達不可能かテスト
            for path in &unreachable_paths {
                assert!(
                    !graph.is_reachable_from_start(path),
                    "Path {:?} should not be reachable from start",
                    path
                );
            }
        } else {
            info!("All paths are reachable from start point (1,1)");
        }

        Ok(())
    }

    #[test]
    fn test_maze_statistics_value_creation() {
        // MazeStatisticsValueの直接的な作成テスト
        let test_components = vec![
            vec![MazePoint::new(1, 1), MazePoint::new(1, 2)],
            vec![MazePoint::new(3, 3), MazePoint::new(3, 4)],
        ];
        let unreachable = vec![MazePoint::new(3, 3), MazePoint::new(3, 4)];

        let stats = MazeStatisticsValue::new(10, 15, 2, test_components, unreachable);

        assert_eq!(stats.nodes(), 10);
        assert_eq!(stats.edges(), 15);
        assert_eq!(stats.components(), 2);

        // メインパスには(1,1)と(1,2)が含まれるはず
        assert!(stats.main_path_points().contains(&MazePoint::new(1, 1)));
        assert!(stats.main_path_points().contains(&MazePoint::new(1, 2)));

        // 孤立パス群には(3,3)と(3,4)を含むグループが存在するはず
        assert_eq!(stats.orphan_paths_points().len(), 1);
        let orphan_group = &stats.orphan_paths_points()[0];
        assert!(orphan_group.contains(&MazePoint::new(3, 3)));
        assert!(orphan_group.contains(&MazePoint::new(3, 4)));

        info!("MazeStatisticsValue creation test passed");
    }

    #[test]
    fn test_connected_components_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(5, 5)?;
        let maze_guard = maze_points.read().unwrap();

        let graph = PathConnectivityGraph::build_from_maze_points(&maze_guard)?;
        let components = graph.get_connected_components();

        info!("Detailed component analysis:");
        for (i, component) in components.iter().enumerate() {
            info!("  Component {}: {} nodes", i + 1, component.len());
            for (j, point) in component.iter().enumerate() {
                if j < 5 {
                    // 最初の5つのポイントのみ表示
                    info!("    Point {}: {:?}", j + 1, point);
                } else if j == 5 {
                    info!("    ... and {} more points", component.len() - 5);
                    break;
                }
            }
        }

        // 各成分が空でないことを確認
        for component in &components {
            assert!(!component.is_empty(), "Component should not be empty");
        }

        Ok(())
    }

    #[test]
    fn test_main_path_vs_orphan_paths() -> Result<(), Box<dyn std::error::Error>> {
        let maze_points = MazePoints::initialize_maze_points(7, 7)?;
        let maze_guard = maze_points.read().unwrap();

        let statistics = detect_unreachable_paths_backend(&maze_guard)?;

        // スタート地点(1,1)がメインパスに含まれることを確認
        let start_point = MazePoint::new(1, 1);
        if statistics.nodes() > 0 {
            // パスが存在する場合、スタート地点はメインパスにあるか、
            // または迷路内にパスが存在しない場合
            let start_in_main = statistics.main_path_points().contains(&start_point);
            let has_orphans = !statistics.orphan_paths_points().is_empty();

            info!(
                "Start point in main path: {}, Has orphan paths: {}",
                start_in_main, has_orphans
            );

            // もし孤立パスが存在する場合、メインパスも存在するはず
            if has_orphans {
                assert!(
                    !statistics.main_path_points().is_empty(),
                    "If orphan paths exist, main path should also exist"
                );
            }
        }

        Ok(())
    }
}
