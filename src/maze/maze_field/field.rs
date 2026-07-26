use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use rand::RngExt;

use crate::maze::{
    maze_cell::{
        maze_point::{
            point::{MazePoint, get_between_points},
            point_status::MazePointStatus,
        },
        wall::wall_identifier::WallIdentifier,
    },
    maze_field::extend_result::ExtendResult,
};

/// 迷路の全座標点・柱・外壁などの状態を管理する構造体。
///
/// - 各座標点の状態（道・壁・柱・外壁）を保持し、迷路生成アルゴリズムの進行を管理します。
/// - スレッドセーフな迷路生成のためのデータ構造です。
/// - 新しい `MazePointStatus` のメソッドを活用し、安全で保守しやすい実装です。
#[derive(Debug, Clone)]
pub(crate) struct Field {
    /// 迷路のX方向の大きさ
    x_size: u64,

    /// 迷路のY方向の大きさ
    y_size: u64,

    /// 迷路の全座標点とその状態のマッピング
    all_maze_points: HashMap<MazePoint, MazePointStatus>,

    /// 迷路の柱の座標集合
    /// 境界に接していない内部の偶数座標のみが含まれる
    pillar_points: HashSet<MazePoint>,
    /// 拡張処理中の柱の座標集合
    extending_pillar_points: HashSet<MazePoint>,

    /// 生成起点の外壁の座標集合
    /// 0<x<x_size,0<y<y_sizeのxyどちらかが偶数座標の外壁のみが含まれる。
    extend_start_points: HashSet<MazePoint>,
    /// 拡張処理中生成起点の外壁の座標集合
    extending_start_points: HashSet<MazePoint>,

    /// 区画分割数（1以上）
    partition_count: usize,

    /// 区画ごとの未探索開始点キュー
    available_start_points_by_partition: Vec<Vec<MazePoint>>,

    /// 区画ごとの拡張可能な柱候補キュー
    available_extending_pillars_by_partition: Vec<Vec<MazePoint>>,
}

impl Field {
    /// 迷路のX方向の大きさを返します。
    pub fn x_size(&self) -> u64 {
        self.x_size
    }

    /// 迷路のY方向の大きさを返します。
    pub fn y_size(&self) -> u64 {
        self.y_size
    }

    /// 迷路の全座標点とその状態のクローンを返します。
    pub fn get_all_maze_points_clone(&self) -> HashMap<MazePoint, MazePointStatus> {
        self.all_maze_points.clone()
    }

    #[allow(dead_code)]
    pub fn get_maze_point_status(&self, point: &MazePoint) -> Option<MazePointStatus> {
        self.all_maze_points.get(point).cloned()
    }

    /// 迷路の柱座標集合のクローンを返します。
    pub fn get_pillar_points_clone(&self) -> HashSet<MazePoint> {
        self.pillar_points.clone()
    }

    /// 拡張処理中の柱座標集合のクローンを返します。
    pub fn get_extending_pillar_points_clone(&self) -> HashSet<MazePoint> {
        self.extending_pillar_points.clone()
    }

    #[allow(dead_code)]
    pub fn get_available_pillar_points(&self) -> Vec<MazePoint> {
        // pillar_pointsに含まれるが、extending_pillar_pointsに含まれない柱をフィルタリング
        self.pillar_points
            .iter()
            .filter(|point| !self.extending_pillar_points.contains(*point))
            .cloned()
            .collect()
    }

    pub fn get_extend_start_point_clone(&self) -> HashSet<MazePoint> {
        self.extend_start_points.clone()
    }

    pub fn get_extending_start_points_clone(&self) -> HashSet<MazePoint> {
        self.extending_start_points.clone()
    }

    fn normalized_partition_count(partition_count: usize) -> usize {
        if partition_count == 0 {
            1
        } else {
            partition_count
        }
    }

    fn partition_id_for_point_with_count(&self, point: &MazePoint, partition_count: usize) -> usize {
        if partition_count <= 1 {
            return 0;
        }

        let width = self.x_size as usize;
        if width <= 1 {
            return 0;
        }

        let x = point.x() as usize;
        if x == 0 {
            return 0;
        }
        if x >= width - 1 {
            return partition_count - 1;
        }

        let mut pid = (x * partition_count) / width;
        if pid >= partition_count {
            pid = partition_count - 1;
        }
        pid
    }

    fn partition_id_for_point(&self, point: &MazePoint) -> usize {
        self.partition_id_for_point_with_count(point, self.partition_count)
    }

    fn point_owned_by_partition(&self, point: &MazePoint, partition_id: usize) -> bool {
        self.partition_id_for_point(point) == partition_id
    }

    pub(crate) fn configure_partitions(&mut self, partition_count: usize) {
        self.partition_count = Self::normalized_partition_count(partition_count);
        self.rebuild_partition_queues();
    }

    fn rebuild_partition_queues(&mut self) {
        self.available_start_points_by_partition = vec![Vec::new(); self.partition_count];
        self.available_extending_pillars_by_partition = vec![Vec::new(); self.partition_count];

        for point in &self.extend_start_points {
            if self.extending_start_points.contains(point) {
                continue;
            }
            let partition_id = self.partition_id_for_point(point);
            self.available_start_points_by_partition[partition_id].push(*point);
        }

        for point in &self.extending_pillar_points {
            let partition_id = self.partition_id_for_point(point);
            self.available_extending_pillars_by_partition[partition_id].push(*point);
        }
    }

    fn pop_random_start_point_in_partition(&mut self, partition_id: usize) -> Option<MazePoint> {
        if partition_id >= self.partition_count {
            return None;
        }

        let mut rng = rand::rng();
        let bucket = &mut self.available_start_points_by_partition[partition_id];

        while !bucket.is_empty() {
            let idx = rng.random_range(0..bucket.len());
            let point = bucket.swap_remove(idx);

            if self.extending_start_points.contains(&point) {
                continue;
            }
            if !self.extend_start_points.contains(&point) {
                continue;
            }

            if let Some(status) = self.all_maze_points.get(&point)
                && status.is_not_checked_start_point()
            {
                return Some(point);
            }
        }

        None
    }

    fn pop_random_extending_source_pillar_in_partition(
        &mut self,
        partition_id: usize,
    ) -> Option<MazePoint> {
        if partition_id >= self.partition_count {
            return None;
        }

        let mut rng = rand::rng();

        loop {
            let point = {
                let bucket = &mut self.available_extending_pillars_by_partition[partition_id];
                if bucket.is_empty() {
                    return None;
                }
                let idx = rng.random_range(0..bucket.len());
                bucket.swap_remove(idx)
            };

            if !self.extending_pillar_points.contains(&point) {
                continue;
            }

            if let Some(status) = self.all_maze_points.get(&point)
                && status.is_extending_pillar()
                && !self
                    .get_adjacent_extendable_pillars_for_partition(&point, partition_id)
                    .is_empty()
            {
                return Some(point);
            }
        }
    }

    pub(in crate::maze) fn select_start_source_for_partition(
        &mut self,
        partition_id: usize,
        identifier: &WallIdentifier,
    ) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
        if partition_id >= self.partition_count {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Invalid partition id {}. partition_count={}.",
                    partition_id, self.partition_count
                ),
            )));
        }

        loop {
            if self.all_pillar_seeked_flag() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "All start points have been sought.",
                )));
            }

            if let Some(start_point) = self.pop_random_start_point_in_partition(partition_id) {
                if self
                    .mark_start_point_as_extending(&start_point, identifier)
                    .is_ok()
                {
                    return Ok(start_point);
                }
                continue;
            }

            if let Some(source_pillar) =
                self.pop_random_extending_source_pillar_in_partition(partition_id)
            {
                if self
                    .mark_extending_pillar_with_identifier(&source_pillar, identifier)
                    .is_ok()
                {
                    self.available_extending_pillars_by_partition[partition_id].push(source_pillar);
                    return Ok(source_pillar);
                }
                continue;
            }

            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "No available start point or extending pillar source found in partition {}",
                    partition_id
                ),
            )));
        }
    }
    #[allow(dead_code)]
    pub fn get_random_available_pillar_point(&self) -> Option<MazePoint> {
        let available_points = self.get_available_pillar_points();
        if available_points.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(available_points[rng.random_range(0..available_points.len())])
        }
    }

    pub fn get_available_start_points(&self) -> Vec<MazePoint> {
        // extend_start_pointsに含まれるが、extending_start_pointsに含まれない柱をフィルタリング
        self.extend_start_points
            .iter()
            .filter(|point| !self.extending_start_points.contains(*point))
            .cloned()
            .collect()
    }

    /// 利用可能な外壁開始点からランダムに1つ選択します。
    pub fn get_random_available_start_point(&self) -> Option<MazePoint> {
        let available_points = self.get_available_start_points();
        if available_points.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(available_points[rng.random_range(0..available_points.len())])
        }
    }

    pub fn get_available_extending_source_pillars(&self) -> Vec<MazePoint> {
        self.extending_pillar_points
            .iter()
            .filter(|point| !self.get_adjacent_extendable_pillars(point).is_empty())
            .cloned()
            .collect()
    }

    pub fn get_random_available_extending_source_pillar(&self) -> Option<MazePoint> {
        let available_points = self.get_available_extending_source_pillars();
        if available_points.is_empty() {
            None
        } else {
            let mut rng = rand::rng();
            Some(available_points[rng.random_range(0..available_points.len())])
        }
    }

    pub fn get_adjacent_extendable_pillars(&self, source_point: &MazePoint) -> Vec<MazePoint> {
        let mut adjacent_pillars = source_point.generate_adjacent_maze_points(2);
        // extend_start_pointsに含まれる開始点とextending_pillar_pointsに含まれる柱を除外する。
        adjacent_pillars.retain(|point| {
            !self.extend_start_points.contains(point)
                && !self.extending_pillar_points.contains(point)
                && self.pillar_points.contains(point)
        });
        adjacent_pillars
    }

    pub fn get_adjacent_extendable_pillars_for_partition(
        &self,
        source_point: &MazePoint,
        partition_id: usize,
    ) -> Vec<MazePoint> {
        let mut adjacent_pillars = self.get_adjacent_extendable_pillars(source_point);
        adjacent_pillars.retain(|point| self.point_owned_by_partition(point, partition_id));
        adjacent_pillars
    }

    /// すべての柱が探索済みならtrueを返します。
    pub fn all_pillar_seeked_flag(&self) -> bool {
        self.pillar_points.len() == self.extending_pillar_points.len()
    }

    /// 指定サイズの迷路データを初期化し、スレッドセーフなラッパーで返します。
    ///
    /// # 引数
    /// * `x_size` - X方向サイズ（5以上の奇数）
    /// * `y_size` - Y方向サイズ（5以上の奇数）
    ///
    /// # 戻り値
    /// * `Ok(Arc<RwLock<Field>>)` - 初期化済み迷路データ
    /// * `Err(_)` - サイズ不正等
    ///
    /// # エラー条件
    /// - サイズが5未満または偶数
    /// - i32の範囲外
    ///
    /// # 使用例
    /// ```rust
    /// let maze = Field::initialize_maze_points(7, 7)?;
    /// ```
    pub fn initialize_maze_points(
        x_size: u64,
        y_size: u64,
    ) -> Result<Arc<RwLock<Self>>, Box<dyn std::error::Error>> {
        if x_size < 5 || y_size < 5 || x_size % 2 == 0 || y_size % 2 == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "x_size and y_size must be odd numbers and >= 5",
            )));
        }

        // 引数がi32の範囲内の値であることを確認する
        if x_size > i32::MAX as u64 || y_size > i32::MAX as u64 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "x_size and y_size must be within the range of i32",
            )));
        }

        // 全座標をPathで初期化
        let mut all_maze_points: HashMap<MazePoint, MazePointStatus> = HashMap::new();
        for y in 0..y_size {
            for x in 0..x_size {
                let point = MazePoint::new(x as u64, y as u64);
                all_maze_points.insert(point, MazePointStatus::new_not_resolved_path());
            }
        }

        let mut extend_start_points = HashSet::new();
        let outside_wall_id = WallIdentifier::new();

        // 境界を外壁に設定（新しいコンストラクタメソッドを使用）
        for y in 0..y_size {
            for x in 0..x_size {
                if x == 0 || x == x_size - 1 || y == 0 || y == y_size - 1 {
                    // 0<x<x_size,0<y<y_sizeのxyのどちらかが偶数座標の外壁はstart_point。
                    if !(x == 0 && y == 0 || x == x_size - 1 && y == y_size - 1)
                        && (x % 2 == 0 || y % 2 == 0)
                    {
                        // 偶数座標の外壁はstart_point
                        all_maze_points.insert(
                            MazePoint::new(x as u64, y as u64),
                            MazePointStatus::new_start_point_outside_wall(outside_wall_id),
                        );
                        extend_start_points.insert(MazePoint::new(x as u64, y as u64));
                    } else {
                        // 偶数座標以外の外壁はjust_outside_wall
                        all_maze_points.insert(
                            MazePoint::new(x as u64, y as u64),
                            MazePointStatus::new_just_outside_wall(outside_wall_id),
                        );
                    }
                }
            }
        }

        let mut pillar_points = HashSet::new();
        // 偶数座標に柱を配置（新しいコンストラクタメソッドを使用）
        for y in 0..y_size {
            for x in 0..x_size {
                if x % 2 == 0 && y % 2 == 0 {
                    let point = MazePoint::new(x as u64, y as u64);
                    if let Some(point_value) = all_maze_points.get_mut(&point)
                        && point_value.is_path()
                    {
                        // PathであればNotChecked柱に変更
                        *point_value = MazePointStatus::new_notchecked_pillar();

                        // 境界に接していない内部の柱のみを壁生成開始点として追加
                        if x > 0 && y > 0 && x < x_size - 1 && y < y_size - 1 {
                            pillar_points.insert(point);
                        }
                    }
                }
            }
        }

        Ok(Arc::new(RwLock::new(Field {
            x_size,
            y_size,
            all_maze_points,
            pillar_points,
            extending_pillar_points: HashSet::new(),
            extend_start_points,
            extending_start_points: HashSet::new(),
            partition_count: 1,
            available_start_points_by_partition: Vec::new(),
            available_extending_pillars_by_partition: Vec::new(),
        })))
    }

    /// 指定した外壁開始点をExtending状態に変更します。
    ///
    /// # 引数
    /// * `start_point` - 拡張中にする外壁開始点
    /// * `identifier` - 壁の識別子
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(_)` - 状態変換失敗等
    pub(in crate::maze) fn mark_start_point_as_extending(
        &mut self,
        start_point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // すでに拡張中の生成起点であればエラー
        if self.extending_start_points.contains(start_point) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Start point {:?} is already extending", start_point),
            )));
        }

        if let Some(current_status) = self.all_maze_points.get(start_point).cloned() {
            // 新しいメソッドを使用して状態変換
            let extending_status =
                MazePointStatus::new_extending_start_point_from_notchecked_start_point(
                    current_status,
                    identifier,
                )
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Failed to convert start point to extending state at {:?}: {}",
                            start_point, e
                        ),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

            // 状態を更新し、拡張中生成起点セットに追加
            self.all_maze_points.insert(*start_point, extending_status);
            self.extending_start_points.insert(*start_point);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Start point {:?} not found", start_point),
            )))
        }
    }

    /// 指定した柱をExtending状態に変更します（可能な場合のみ）。
    ///
    /// # 引数
    /// * `point` - 柱座標
    /// * `identifier` - 壁の識別子
    ///
    /// # 戻り値
    /// * `Ok(true)` - 状態変更成功
    /// * `Ok(false)` - 既に拡張済み等で変更なし
    /// * `Err(_)` - 状態変換失敗
    fn mark_pillar_as_extending(
        &mut self,
        point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // 拡張済みでなく、新しいメソッドを使用してNotChecked状態であることを確認
        if !self.extending_pillar_points.contains(point)
            && let Some(status) = self.all_maze_points.get(point).cloned()
            && status.is_not_checked_pillar()
        {
            // 新しいメソッドを使用して状態変換
            let extending_status =
                MazePointStatus::new_extending_pillar_from_notchecked_pillar(status, identifier)
                    .map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to convert pillar to extending state: {}", e),
                        )) as Box<dyn std::error::Error + Send + Sync>
                    })?;

            self.all_maze_points.insert(*point, extending_status);
            self.extending_pillar_points.insert(*point);
            let partition_id = self.partition_id_for_point(point);
            if partition_id < self.available_extending_pillars_by_partition.len() {
                self.available_extending_pillars_by_partition[partition_id].push(*point);
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub(in crate::maze) fn mark_extending_pillar_with_identifier(
        &mut self,
        point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.extending_pillar_points.contains(point) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Point {:?} is not an extending pillar", point),
            )));
        }

        if let Some(status) = self.all_maze_points.get(point).cloned() {
            if !status.is_extending_pillar() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Point {:?} does not have extending pillar status", point),
                )));
            }

            let updated_status = MazePointStatus::add_wall_identifier(status, identifier)
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to append wall identifier at {:?}: {}", point, e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

            self.all_maze_points.insert(*point, updated_status);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Point {:?} not found", point),
            )))
        }
    }

    /// 指定した中間点をWall状態に変更します（Path状態のみ）。
    ///
    /// # 引数
    /// * `point` - 中間点座標
    /// * `identifier` - 壁の識別子
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(_)` - Pathでない等
    fn path_to_wall(
        &mut self,
        point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(status) = self.all_maze_points.get(point) {
            match status {
                MazePointStatus::Path(..) => {
                    self.all_maze_points
                        .insert(*point, MazePointStatus::new_maze_wall(*identifier));
                    Ok(())
                }
                _ => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Middle point {:?} is not a Path", point),
                ))),
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Middle point {:?} not found", point),
            )))
        }
    }

    /// 柱または外壁開始点から隣接柱への拡張処理を実行します（原子的操作）。
    ///
    /// # 引数
    /// * `from_point` - 拡張元の柱または外壁開始点
    /// * `to_pillar` - 拡張先の柱
    /// * `identifier` - 壁の識別子
    ///
    /// # 戻り値
    /// * `Ok(ExtendResult)` - 拡張結果
    /// * `Err(_)` - 状態不正・中間点不正等
    pub(in crate::maze) fn execute_pillar_extension(
        &mut self,
        from_point: &MazePoint,
        to_pillar: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<ExtendResult, Box<dyn std::error::Error + Send + Sync>> {
        let from_point_status = self
            .all_maze_points
            .get(from_point)
            .cloned()
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("From point {:?} not found", from_point),
                ))
            })?;

        let to_pillar_status = self
            .all_maze_points
            .get(to_pillar)
            .cloned()
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("To pillar {:?} not found", to_pillar),
                ))
            })?;

        // from_point_statusがOutsideもしくはPillarではない場合はエラー。
        // from_point_statusがOutsideの場合は、StartPointではない場合はエラー。
        // from_point_statusがExtendingではない場合はエラー。
        if !from_point_status.is_extending_start_point() && !from_point_status.is_extending_pillar()
        {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "From point {:?} is not outside or pillar, or not an Extending point {:?}",
                    from_point, from_point_status
                ),
            )));
        }

        // to_pillar_statusがPillarではない場合はエラー。
        // to_pillar_statusがNotCheckedではない場合はエラー。
        if !to_pillar_status.is_not_checked_pillar() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "To pillar {:?} is not a NotChecked Pillar {:?}",
                    to_pillar, to_pillar_status
                ),
            )));
        }

        // 中間点を計算
        let middle_points = get_between_points(from_point, to_pillar);

        // 中間点が正確に1つであることを確認
        if middle_points.len() != 3 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Expected exactly 3 points between {:?} and {:?}, but got {} points: {:?}",
                    from_point,
                    to_pillar,
                    middle_points.len(),
                    middle_points
                ),
            )));
        }

        // 中間点を取得（3つの点のうち真ん中の点）
        let middle_point = middle_points[1];

        // from_pointとto_pillarが含まれていることを確認
        if !middle_points.contains(from_point) || !middle_points.contains(to_pillar) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "get_between_points result does not contain expected pillars. From: {:?}, To: {:?}, Points: {:?}",
                    from_point, to_pillar, middle_points
                ),
            )));
        }

        // 中間点を壁に変更
        self.path_to_wall(&middle_point, identifier)?;

        // 拡張先の状態を確認して適切に処理
        if let Some(status) = self.all_maze_points.get(to_pillar).cloned() {
            if status.is_outside_wall() {
                Ok(ExtendResult::new_outside())
            } else if status.is_not_checked_pillar() {
                // 拡張先がNotChecked状態の柱であれば、拡張処理を行う
                self.mark_pillar_as_extending(to_pillar, identifier)?;
                Ok(ExtendResult::new_next_pillar(*to_pillar))
            } else if status.is_extending_pillar() {
                // 既に拡張中の柱であれば、何もしない
                Ok(ExtendResult::new_extending_pillar())
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unexpected pillar status for {:?}: {:?}", to_pillar, status),
                )))
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Selected point {:?} not found", to_pillar),
            )))
        }
    }
}
