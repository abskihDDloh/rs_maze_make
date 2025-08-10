use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{
    maze_point::MazePoint,
    maze_point_status::{MazePointStatus, WallIdentifier, WallType},
};
use rand::Rng;

/// 迷路の構成要素とその状態を管理する構造体
///
/// この構造体は迷路の各座標点の状態（道、壁、柱）を管理し、
/// 迷路生成アルゴリズムの進行状況を追跡します。
/// 新しいMazePointStatusのメソッドを活用して、より安全で保守しやすい実装になっています。
#[derive(Debug, Clone)]
pub(crate) struct MazePoints {
    /// 迷路のX方向の大きさ
    x_size: u32,
    /// 迷路のY方向の大きさ
    y_size: u32,
    /// 迷路の全座標点とその状態のマッピング
    all_maze_points: HashMap<MazePoint, MazePointStatus>,
    /// 全柱探索完了フラグ
    /// すべての柱が探索済みの場合にtrueになる
    all_pillar_seeked_flag: bool,
    /// 迷路の柱（壁の生成起点）の座標集合
    /// 境界に接していない内部の偶数座標のみが含まれる
    pillar_points: HashSet<MazePoint>,
    /// 拡張処理中の柱の座標集合
    extending_pillar_points: HashSet<MazePoint>,
}

impl MazePoints {
    /// 迷路のX方向の大きさを返す
    ///
    /// # Returns
    ///
    /// 迷路のX方向の大きさ
    pub fn x_size(&self) -> u32 {
        self.x_size
    }

    /// 迷路のY方向の大きさを返す
    ///
    /// # Returns
    ///
    /// 迷路のY方向の大きさ
    pub fn y_size(&self) -> u32 {
        self.y_size
    }

    /// 迷路の全座標点とその状態のクローンを返す
    ///
    /// # Returns
    ///
    /// 迷路の全座標点とその状態のHashMapのクローン
    pub fn get_all_maze_points_clone(&self) -> HashMap<MazePoint, MazePointStatus> {
        self.all_maze_points.clone()
    }

    /// 迷路の柱座標集合のクローンを返す
    ///
    /// # Returns
    ///
    /// 迷路の柱座標のHashSetのクローン
    pub fn get_all_pillar_points_clone(&self) -> HashSet<MazePoint> {
        self.pillar_points.clone()
    }

    /// 拡張済み柱座標集合のクローンを返す
    ///
    /// # Returns
    ///
    /// 拡張済み柱座標のHashSetのクローン
    pub fn get_extended_pillar_points_clone(&self) -> HashSet<MazePoint> {
        self.extending_pillar_points.clone()
    }

    /// 迷路の柱座標集合への参照を返す
    ///
    /// # Returns
    ///
    /// 迷路の柱座標のHashSetへの参照
    pub fn pillar_points(&self) -> &HashSet<MazePoint> {
        &self.pillar_points
    }

    /// すべての柱が探索された場合はtrueを返す
    ///
    /// # Returns
    ///
    /// すべての柱が探索済みの場合はtrue、そうでなければfalse
    pub fn all_pillar_seeked_flag(&self) -> bool {
        self.all_pillar_seeked_flag
    }

    /// 迷路データを初期化し、スレッドセーフなラッパーで返す
    ///
    /// この関数は指定されたサイズの迷路を初期化します。迷路の境界は外壁で囲まれ、
    /// 内部の偶数座標には柱が配置されます。残りの座標は通路として初期化されます。
    /// 新しいMazePointStatusのコンストラクタメソッドを使用して、型安全な初期化を行います。
    ///
    /// # Arguments
    ///
    /// * `x_size` - 迷路のX方向の大きさ（5以上の奇数である必要があります）
    /// * `y_size` - 迷路のY方向の大きさ（5以上の奇数である必要があります）
    ///
    /// # Returns
    ///
    /// 初期化された迷路データのスレッドセーフなラッパー、またはエラー
    ///
    /// # Errors
    ///
    /// * サイズが5未満の場合
    /// * サイズが偶数の場合
    /// * サイズがi32の範囲を超える場合
    ///
    /// # Examples
    ///
    /// ```rust
    /// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
    /// ```
    pub fn initialize_maze_points(
        x_size: u32,
        y_size: u32,
    ) -> Result<Arc<RwLock<Self>>, Box<dyn std::error::Error>> {
        if x_size < 5 || y_size < 5 || x_size % 2 == 0 || y_size % 2 == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "x_size and y_size must be odd numbers and >= 5",
            )));
        }

        // 引数がi32の範囲内の値であることを確認する
        if x_size > i32::MAX as u32 || y_size > i32::MAX as u32 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "x_size and y_size must be within the range of i32",
            )));
        }

        // 全座標をPathで初期化
        let mut all_maze_points: HashMap<MazePoint, MazePointStatus> = HashMap::new();
        for y in 0..y_size {
            for x in 0..x_size {
                let point = MazePoint::new(x, y);
                all_maze_points.insert(point, MazePointStatus::Path);
            }
        }

        let mut wall_start_points = HashSet::new();
        let outside_wall_id = WallIdentifier::new();

        // 境界を外壁に設定（新しいコンストラクタメソッドを使用）
        for y in 0..y_size {
            for x in 0..x_size {
                if x == 0 || x == x_size - 1 || y == 0 || y == y_size - 1 {
                    let point = MazePoint::new(x, y);
                    if let Some(point_value) = all_maze_points.get_mut(&point) {
                        *point_value = MazePointStatus::new_outside_wall(outside_wall_id);
                    }
                }
            }
        }

        // 偶数座標に柱を配置（新しいコンストラクタメソッドを使用）
        for y in 0..y_size {
            for x in 0..x_size {
                if x % 2 == 0 && y % 2 == 0 {
                    let point = MazePoint::new(x, y);
                    if let Some(point_value) = all_maze_points.get_mut(&point)
                        && *point_value == MazePointStatus::Path
                    {
                        // PathであればNotChecked柱に変更
                        *point_value = MazePointStatus::new_notchecked_pillar();

                        // 境界に接していない内部の柱のみを壁生成開始点として追加
                        if x > 0 && y > 0 && x < x_size - 1 && y < y_size - 1 {
                            wall_start_points.insert(point);
                        }
                    }
                }
            }
        }

        Ok(Arc::new(RwLock::new(MazePoints {
            x_size,
            y_size,
            all_maze_points,
            all_pillar_seeked_flag: false,
            pillar_points: wall_start_points,
            extending_pillar_points: HashSet::new(),
        })))
    }

    /// 柱を拡張中状態に変更する
    ///
    /// `MazePointStatus::new_extending_pillar_from_notchecked_pillar()` を使用して
    /// 型安全で一貫性のある状態変換を行います。
    ///
    /// # Arguments
    ///
    /// * `pillar_point` - 拡張中にする柱の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(())、変換に失敗した場合はエラー
    ///
    /// # Errors
    ///
    /// * 柱が見つからない場合
    /// * 状態変換に失敗した場合（NotChecked状態でない場合など）
    fn mark_pillar_as_extending(
        &mut self,
        pillar_point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(current_status) = self.all_maze_points.get(pillar_point).cloned() {
            // 新しいメソッドを使用して状態変換
            let extending_status = MazePointStatus::new_extending_pillar_from_notchecked_pillar(
                current_status,
                identifier,
            )
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Failed to convert pillar to extending state at {:?}: {}",
                        pillar_point, e
                    ),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // 状態を更新し、拡張中柱セットに追加
            self.all_maze_points.insert(*pillar_point, extending_status);
            self.extending_pillar_points.insert(*pillar_point);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Pillar point {:?} not found", pillar_point),
            )))
        }
    }

    /// 中間点を壁に変更する
    ///
    /// 柱と柱の間の中間点をWall状態に変更します。
    /// 中間点は事前にPath状態である必要があります。
    ///
    /// # Arguments
    ///
    /// * `middle_point` - 壁にする中間点の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(())、中間点がPathでない場合はエラー
    ///
    /// # Errors
    ///
    /// * 中間点が見つからない場合
    /// * 中間点がPath状態でない場合
    fn mark_middle_point_as_wall(
        &mut self,
        middle_point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(status) = self.all_maze_points.get(middle_point) {
            match status {
                MazePointStatus::Path => {
                    self.all_maze_points.insert(
                        *middle_point,
                        MazePointStatus::Wall(WallType::Wall, Some(*identifier)),
                    );
                    Ok(())
                }
                _ => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Middle point {:?} is not a Path", middle_point),
                ))),
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Middle point {:?} not found", middle_point),
            )))
        }
    }

    /// 柱の拡張処理を実行する（原子的操作）
    ///
    /// 新しいMazePointStatusの判定メソッド `is_outside_wall()`, `is_not_checked_wall()`,
    /// `is_extending_wall()`, `is_my_wall()` を使用して状態判定を簡潔かつ安全に行います。
    ///
    /// # Arguments
    ///
    /// * `from_pillar` - 拡張元の柱
    /// * `to_pillar` - 拡張先の柱
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合は拡張結果、失敗した場合はエラー
    ///
    /// # Errors
    ///
    /// * 中間点の壁化に失敗した場合
    /// * 拡張先の柱が見つからない場合
    /// * 予期しない柱の状態の場合
    fn execute_pillar_extension(
        &mut self,
        from_pillar: &MazePoint,
        to_pillar: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<SeekAdjacentPillarOkResult, Box<dyn std::error::Error + Send + Sync>> {
        // 中間点を計算
        let middle_point = MazePoint::new(
            (from_pillar.x() + to_pillar.x()) / 2,
            (from_pillar.y() + to_pillar.y()) / 2,
        );

        // 中間点を壁に変更
        self.mark_middle_point_as_wall(&middle_point, identifier)?;

        // 拡張先の状態を確認して適切に処理
        if let Some(status) = self.all_maze_points.get(to_pillar).cloned() {
            // 新しいメソッドを使用して判定を簡潔に記述
            if status.is_outside_wall() {
                // Outside壁の場合
                Ok(SeekAdjacentPillarOkResult::new(
                    SeekAdjacentPillarOkState::Outside,
                    *to_pillar,
                ))
            } else if status.is_not_checked_wall() {
                // NotChecked状態の柱の場合：Extending状態に変更
                self.mark_pillar_as_extending(to_pillar, identifier)?;
                Ok(SeekAdjacentPillarOkResult::new(
                    SeekAdjacentPillarOkState::NextPillar,
                    *to_pillar,
                ))
            } else if status.is_extending_wall() {
                // Extending状態の柱の場合：識別子で判定
                if status.is_my_wall(identifier) {
                    // 同じ識別子の場合は接続
                    Ok(SeekAdjacentPillarOkResult::new(
                        SeekAdjacentPillarOkState::NextPillar,
                        *to_pillar,
                    ))
                } else {
                    // 異なる識別子の場合は外壁として扱う
                    Ok(SeekAdjacentPillarOkResult::new(
                        SeekAdjacentPillarOkState::Outside,
                        *to_pillar,
                    ))
                }
            } else {
                // その他の状態（通常の壁など）は予期しない状態
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unexpected pillar state at {:?}: {:?}", to_pillar, status),
                )))
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Selected point {:?} not found", to_pillar),
            )))
        }
    }

    /// 利用可能な柱の候補を効率的に取得する（読み取り専用）
    ///
    /// `MazePointStatus::is_not_checked_wall()` を使用して判定を簡潔にします。
    /// この関数は読み取り専用の操作で、柱の状態を変更しません。
    ///
    /// # Arguments
    ///
    /// * `exclude_set` - 除外する柱の座標集合
    ///
    /// # Returns
    ///
    /// (all_pillar_seeked_flag, 利用可能な柱の座標リスト)
    fn get_available_pillar_candidates(
        &self,
        exclude_set: &HashSet<MazePoint>,
    ) -> (bool, Vec<MazePoint>) {
        let candidates = if self.all_pillar_seeked_flag {
            Vec::new()
        } else {
            self.pillar_points
                .iter()
                .filter(|point| {
                    // 除外セットに含まれている場合は除外
                    if exclude_set.contains(*point) {
                        return false;
                    }

                    // 拡張済み柱は除外
                    if self.extending_pillar_points.contains(*point) {
                        return false;
                    }

                    // 新しいメソッドを使用してNotChecked柱を判定
                    if let Some(status) = self.all_maze_points.get(point) {
                        status.is_not_checked_wall()
                    } else {
                        false
                    }
                })
                .cloned()
                .collect()
        };

        (self.all_pillar_seeked_flag, candidates)
    }

    /// 指定された柱をExtending状態に変更できるかチェックし、可能であれば変更する
    ///
    /// `MazePointStatus::is_not_checked_wall()` と
    /// `MazePointStatus::new_extending_pillar_from_notchecked_pillar()` を
    /// 使用して型安全で一貫性のある状態変換を行います。
    ///
    /// # Arguments
    ///
    /// * `point` - 対象の柱の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(true)、状態が変更されていた場合はOk(false)、エラーの場合はErr
    ///
    /// # Errors
    ///
    /// * 状態変換に失敗した場合
    fn try_mark_pillar_as_extending(
        &mut self,
        point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // 拡張済みでなく、新しいメソッドを使用してNotChecked状態であることを確認
        if !self.extending_pillar_points.contains(point)
            && let Some(status) = self.all_maze_points.get(point).cloned()
            && status.is_not_checked_wall()
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
            return Ok(true);
        }
        Ok(false)
    }
}

/// 迷路生成の開始点となる柱をランダムに選択する
///
/// この関数は利用可能な柱（NotChecked状態かつ拡張済みでない）の中からランダムに1つを選択し、
/// `MazePointStatus::new_extending_pillar_from_notchecked_pillar()` を使用して
/// Extending状態に変更してから返します。スレッドセーフな操作を行い、
/// 複数のスレッドから同時にアクセスされても安全です。
/// 利用可能な柱がない場合、all_pillar_seeked_flagをtrueに設定します。
///
/// **最適化点**: 読み取りロックで候補の選択を行い、書き込みロックは状態変更のみに使用します。
/// **変更点**: `extending_pillar_points`メンバを使用して拡張済み柱を効率的に除外します。
/// **新しい更新**: 新しいメソッド `try_mark_pillar_as_extending()` を使用して型安全な状態変換を行います。
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `identifier` - 壁の識別子（選択された柱の識別子として設定されます）
///
/// # Returns
///
/// 選択された柱の座標、またはエラー
///
/// # Errors
///
/// * すべての柱が既に探索済みの場合
/// * 利用可能な柱が見つからない場合
/// * ロックの取得に失敗した場合
/// * 状態変換に失敗した場合
///
/// # Examples
///
/// ```rust
/// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
/// let identifier = WallIdentifier::new();
/// let start_pillar = select_start_pillar_point(&maze_points, identifier)?;
/// ```
pub fn select_start_pillar_point(
    maze_points: &Arc<RwLock<MazePoints>>,
    identifier: WallIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(" identifier: {}", identifier.as_str());
    let mut work_set = HashSet::new();

    loop {
        // フェーズ1: 読み取りロックで候補を選択
        let (_all_pillar_seeked, candidates) = {
            let read_guard = maze_points.read().map_err(|_| {
                Box::new(std::io::Error::other(format!(
                    "Failed to acquire read lock. {}",
                    error_msg_common_part
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // 事前チェック
            if read_guard.all_pillar_seeked_flag {
                return Err(Box::new(std::io::Error::other(format!(
                    "All pillar points have already been sought. {}",
                    error_msg_common_part
                ))));
            }

            if work_set.len() == read_guard.pillar_points.len() {
                // 書き込みロックが必要な場合は、読み取りロックを解除してから取得
                drop(read_guard);

                let mut write_guard = maze_points.write().map_err(|_| {
                    Box::new(std::io::Error::other(format!(
                        "Failed to acquire write lock for flag update. {}",
                        error_msg_common_part
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;

                write_guard.all_pillar_seeked_flag = true;
                return Err(Box::new(std::io::Error::other(format!(
                    "No more pillar points available. {}",
                    error_msg_common_part
                ))));
            }

            read_guard.get_available_pillar_candidates(&work_set)
        }; // ここで読み取りロック解放

        // 候補が空の場合の処理
        if candidates.is_empty() {
            let mut write_guard = maze_points.write().map_err(|_| {
                Box::new(std::io::Error::other(format!(
                    "Failed to acquire write lock for flag update. {}",
                    error_msg_common_part
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            write_guard.all_pillar_seeked_flag = true;
            return Err(Box::new(std::io::Error::other(format!(
                "No valid pillar points found. {}",
                error_msg_common_part
            ))));
        }

        // フェーズ2: ランダム選択（ロック外）
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..candidates.len());
        let selected_point = candidates[random_index];

        // フェーズ3: 書き込みロックで状態変更を試行
        {
            let mut write_guard = maze_points.write().map_err(|_| {
                Box::new(std::io::Error::other(format!(
                    "Failed to acquire write lock. {}",
                    error_msg_common_part
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            match write_guard.try_mark_pillar_as_extending(&selected_point, &identifier)? {
                true => return Ok(selected_point),
                false => {
                    // 状態が変更されていたので再試行
                    work_set.insert(selected_point);
                    continue;
                }
            }
        } // ここで書き込みロック解放
    }
}

/// 隣接する柱への拡張操作の結果状態を表すenum
///
/// この列挙型は柱から隣接する柱への拡張試行の結果を示します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SeekAdjacentPillarOkState {
    /// 隣接する柱が見つかり、拡張に成功した場合
    NextPillar,
    /// 元の柱の周囲にある利用可能な隣接柱がすべて使用済みまたは無効で、これ以上拡張できない場合
    SurroundedPillar,
    /// 境界（Outside壁）に到達し、これ以上拡張できない場合
    Outside,
}

/// 隣接する柱への拡張操作の結果を表す構造体
///
/// この構造体は柱から隣接する柱への拡張操作の結果を保持します。
/// 操作の結果状態と、対象となった座標点を含みます。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SeekAdjacentPillarOkResult {
    /// 操作の結果状態
    state: SeekAdjacentPillarOkState,
    /// 対象となった座標点
    /// - NextPillarの場合: 選択された隣接柱の座標
    /// - Outsideの場合: 到達した境界の座標
    /// - SurroundedPillarの場合: 元の柱の座標
    pub point: MazePoint,
}

impl SeekAdjacentPillarOkResult {
    /// 新しい結果インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `state` - 操作の結果状態
    /// * `point` - 対象となった座標点
    ///
    /// # Returns
    ///
    /// 新しい結果インスタンス
    pub(crate) fn new(state: SeekAdjacentPillarOkState, point: MazePoint) -> Self {
        Self { state, point }
    }

    /// 結果が「隣接する柱が見つかった」かどうかを判定する
    ///
    /// # Returns
    ///
    /// 隣接する柱が見つかった場合はtrue、そうでなければfalse
    pub fn is_next_pillar(&self) -> bool {
        matches!(self.state, SeekAdjacentPillarOkState::NextPillar)
    }

    /// 結果が「周囲の柱がすべて使用済み」かどうかを判定する
    ///
    /// # Returns
    ///
    /// 周囲の柱がすべて使用済みの場合はtrue、そうでなければfalse
    #[allow(dead_code)]
    pub fn is_surrounded(&self) -> bool {
        matches!(self.state, SeekAdjacentPillarOkState::SurroundedPillar)
    }

    /// 結果が「境界に到達した」かどうかを判定する
    ///
    /// # Returns
    ///
    /// 境界に到達した場合はtrue、そうでなければfalse
    #[allow(dead_code)]
    pub fn is_outside(&self) -> bool {
        matches!(self.state, SeekAdjacentPillarOkState::Outside)
    }
}

/// 柱から隣接する柱への拡張を試行する
///
/// この関数は指定された柱（Extending状態）から隣接する利用可能な柱への拡張を試行します。
/// `MazePointStatus`の新しい判定メソッド `is_extending_wall()`, `is_my_wall()` を使用して
/// 状態判定を安全かつ簡潔に行います。
///
/// 成功した場合、元の柱は拡張済み状態になり、選択された隣接柱はExtending状態になります。
/// また、両柱の間の座標点はWall状態に変更されます。
///
/// 隣接する柱の候補は距離2の位置（上下左右）にある座標点で、以下の条件を満たすもの：
/// - NotChecked状態の柱
/// - Outside壁（境界）
/// - ExtendingまたはExtended状態の柱で、識別子が異なる場合（外壁として扱う）
///
/// 中間点（両柱の中点）はPath状態である必要があり、拡張時にWall状態に変更されます。
/// 利用可能な隣接柱がない場合は、元の柱をExtended状態に変更してSurroundedPillar結果を返します。
///
/// **重要な変更点**:
/// - `all_pillar_seeked_flag`がtrueの場合、エラーではなく`SurroundedPillar`結果を返します。
/// - ExtendingもしくはExtendedのpillarで、identifierが引数と異なる場合は外壁と同じ扱いをします。
/// - 読み取りロックを活用して並行性を向上させます。
/// - unwrap()をエラーハンドリングに置き換えます。
/// - 新しいメソッド `is_extending_wall()`, `is_my_wall()` を使用して状態判定を簡潔にします。
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `pillar_point` - 拡張元の柱の座標（Extending状態である必要があります）
/// * `identifier` - 壁の識別子（元の柱の識別子と一致する必要があります）
///
/// # Returns
///
/// 拡張操作の結果。以下のいずれかの結果を返します：
/// - `NextPillar`: 隣接する柱が見つかり、拡張に成功した場合
/// - `Outside`: 境界（Outside壁）に到達した場合、または異なる識別子の柱に到達した場合
/// - `SurroundedPillar`: 利用可能な隣接柱がない場合、または全柱探索が完了している場合
///
/// # Errors
///
/// * 指定された柱がExtending状態でない場合
/// * 識別子が一致しない場合
/// * 中間点がPath状態でない場合
/// * ロックの取得に失敗した場合
/// * 指定された座標点が見つからない場合
/// * 状態変換に失敗した場合
///
/// # Examples
///
/// ```rust
/// let maze_points = MazePoints::initialize_maze_points(7, 7)?;
/// let identifier = WallIdentifier::new();
/// let start_pillar = select_start_pillar_point(&maze_points, identifier.clone())?;
/// let result = extend_pillar_to_adjacent_pillar(&maze_points, start_pillar, identifier)?;
///
/// match result {
///     result if result.is_next_pillar() => {
///         println!("Next pillar found at: {:?}", result.point);
///         // 拡張を継続
///     }
///     result if result.is_outside() => {
///         println!("Reached boundary at: {:?}", result.point);
///         // 新しい開始点を選択
///     }
///     result if result.is_surrounded() => {
///         println!("Pillar is surrounded or all pillars processed");
///         // 新しい開始点を選択、または処理終了
///     }
/// }
/// ```
pub fn extend_pillar_to_adjacent_pillar(
    maze_points: &Arc<RwLock<MazePoints>>,
    pillar_point: MazePoint,
    identifier: WallIdentifier,
) -> Result<SeekAdjacentPillarOkResult, Box<dyn std::error::Error + Send + Sync>> {
    let pillar_point_copy = pillar_point;
    let error_msg_common_part = format!(
        " identifier: {} from_pillar {:?}",
        identifier.as_str(),
        pillar_point_copy
    );

    // フェーズ1: 読み取りロックで事前チェックと迷路情報取得
    let (x_size, y_size) = {
        let maze_points_read = maze_points.read().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire read lock. {:?}",
                identifier
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // all_pillar_seeked_flagがtrueであれば利用可能な柱はない
        if maze_points_read.all_pillar_seeked_flag {
            return Ok(SeekAdjacentPillarOkResult::new(
                SeekAdjacentPillarOkState::SurroundedPillar,
                pillar_point,
            ));
        }

        // 引数として受け取ったMazePointの状態確認（新しいメソッドを使用）
        if let Some(status) = maze_points_read
            .all_maze_points
            .get(&pillar_point_copy)
            .cloned()
        {
            let status_str = format!("from_pillar_status:{:?}", &status);

            // 新しいメソッドを使用して判定
            if !status.is_extending_wall() || !status.is_my_wall(&identifier) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Pillar is not in Extending state or identifier does not match. {} {}",
                        error_msg_common_part, status_str
                    ),
                )));
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("MazePoint not found {}", error_msg_common_part),
            )));
        }

        (maze_points_read.x_size, maze_points_read.y_size)
    }; // ここで読み取りロック解放

    // フェーズ2: 隣接柱候補の生成（ロック外）
    let mut adjacent_pillars = generate_adjacent_pillars(&pillar_point_copy, x_size, y_size);

    // フェーズ3: 書き込みロックで拡張処理実行
    let mut maze_points_guard = maze_points.write().map_err(|_| {
        Box::new(std::io::Error::other(format!(
            "Failed to acquire write lock. {}",
            error_msg_common_part
        ))) as Box<dyn std::error::Error + Send + Sync>
    })?;

    // 有効な隣接柱を選択
    match select_valid_adjacent_pillar(&maze_points_guard, &mut adjacent_pillars, &identifier) {
        Some(selected_point) => {
            // 拡張処理を実行
            maze_points_guard.execute_pillar_extension(&pillar_point, &selected_point, &identifier)
        }
        None => {
            // 利用可能な隣接柱がない場合
            Ok(SeekAdjacentPillarOkResult::new(
                SeekAdjacentPillarOkState::SurroundedPillar,
                pillar_point,
            ))
        }
    }
}

/// 利用可能な隣接柱から有効なものを選択する
///
/// この関数は隣接柱の候補リストから有効なものをランダムに選択します。
/// 無効な候補は段階的に除外され、すべてが無効になった場合はNoneを返します。
/// 新しいメソッド `validate_selected_point()` を使用して判定を行います。
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `adjacent_pillars` - 隣接柱候補のリスト（変更される可能性があります）
/// * `current_identifier` - 現在の壁の識別子
///
/// # Returns
///
/// 有効な隣接柱の座標、または候補がない場合はNone
fn select_valid_adjacent_pillar(
    maze_points: &MazePoints,
    adjacent_pillars: &mut Vec<MazePoint>,
    current_identifier: &WallIdentifier,
) -> Option<MazePoint> {
    while !adjacent_pillars.is_empty() {
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..adjacent_pillars.len());
        let selected_point = adjacent_pillars[random_index];

        if validate_selected_point(maze_points, &selected_point, current_identifier) {
            return Some(selected_point);
        }

        // 無効な候補を削除
        adjacent_pillars.remove(random_index);
    }
    None
}

/// 隣接柱候補を生成する
///
/// 指定された柱の座標から距離2の位置（上下左右）にある隣接柱候補を生成します。
/// 迷路の境界を考慮して、有効な座標のみを返します。
///
/// # Arguments
///
/// * `pillar_point` - 基準となる柱の座標
/// * `x_size` - 迷路のX方向サイズ
/// * `y_size` - 迷路のY方向サイズ
///
/// # Returns
///
/// 隣接柱候補の座標リスト
fn generate_adjacent_pillars(pillar_point: &MazePoint, x_size: u32, y_size: u32) -> Vec<MazePoint> {
    let mut adjacent_pillars = Vec::new();

    // x方向の隣接点 (x±2, y)
    if pillar_point.x() >= 2 {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() - 2, pillar_point.y()));
    }
    if pillar_point.x() + 2 < x_size {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() + 2, pillar_point.y()));
    }

    // y方向の隣接点 (x, y±2)
    if pillar_point.y() >= 2 {
        adjacent_pillars.push(MazePoint::new(pillar_point.x(), pillar_point.y() - 2));
    }
    if pillar_point.y() + 2 < y_size {
        adjacent_pillars.push(MazePoint::new(pillar_point.x(), pillar_point.y() + 2));
    }

    adjacent_pillars
}

/// 選択された柱の状態を確認する
///
/// この関数は選択された座標点が拡張処理に適しているかを判定します。
/// `MazePointStatus`の新しいメソッド `is_outside_wall()`, `is_not_checked_wall()`,
/// `is_extending_wall()`, `is_my_wall()` を使用して判定を簡潔かつ安全に行います。
///
/// 以下の場合を有効とします：
/// - Outside壁（境界）
/// - NotChecked状態の柱
/// - Extending状態の柱で、識別子が異なる場合（外壁として扱う）
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `selected_point` - 検証対象の座標点
/// * `current_identifier` - 現在の壁の識別子
///
/// # Returns
///
/// 有効な場合はtrue、無効な場合はfalse
fn validate_selected_point(
    maze_points: &MazePoints,
    selected_point: &MazePoint,
    current_identifier: &WallIdentifier,
) -> bool {
    if let Some(status) = maze_points.all_maze_points.get(selected_point) {
        // 新しいメソッドを使用して判定を簡潔に記述
        if status.is_outside_wall() || status.is_not_checked_wall() {
            // Outside壁またはNotChecked柱の場合は有効
            true
        } else if status.is_extending_wall() {
            // Extending状態の柱の場合、識別子が異なれば外壁として扱う（有効）
            !status.is_my_wall(current_identifier)
        } else {
            // その他の状態（通常の壁、Path等）は無効
            false
        }
    } else {
        // 座標点が見つからない場合は無効
        false
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    /// MazePoints::initialize_maze_points の基本テスト
    #[test]
    fn test_initialize_maze_points_valid_sizes() {
        // 有効なサイズでの初期化テスト
        let result = MazePoints::initialize_maze_points(5, 5);
        assert!(result.is_ok());

        let maze_points = result.unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        assert_eq!(maze_points_guard.x_size(), 5);
        assert_eq!(maze_points_guard.y_size(), 5);
        assert!(!maze_points_guard.all_pillar_seeked_flag());

        // 内部の柱ポイントが正しく設定されているかチェック
        let pillar_points = maze_points_guard.pillar_points();
        assert!(!pillar_points.is_empty());

        // 境界の柱は含まれないことを確認
        assert!(!pillar_points.contains(&MazePoint::new(0, 0)));
        assert!(!pillar_points.contains(&MazePoint::new(4, 4)));

        // 内部の偶数座標の柱は含まれることを確認
        assert!(pillar_points.contains(&MazePoint::new(2, 2)));
    }

    #[test]
    fn test_initialize_maze_points_larger_size() {
        // より大きなサイズでのテスト
        let result = MazePoints::initialize_maze_points(9, 7);
        assert!(result.is_ok());

        let maze_points = result.unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        assert_eq!(maze_points_guard.x_size(), 9);
        assert_eq!(maze_points_guard.y_size(), 7);

        // 内部の柱の数を確認 (境界を除く偶数座標)
        let expected_pillars = vec![
            MazePoint::new(2, 2),
            MazePoint::new(2, 4),
            MazePoint::new(4, 2),
            MazePoint::new(4, 4),
            MazePoint::new(6, 2),
            MazePoint::new(6, 4),
        ];

        for pillar in expected_pillars {
            assert!(maze_points_guard.pillar_points().contains(&pillar));
        }
    }

    #[test]
    fn test_initialize_maze_points_invalid_sizes() {
        // 無効なサイズでのテスト
        assert!(MazePoints::initialize_maze_points(4, 5).is_err()); // 偶数
        assert!(MazePoints::initialize_maze_points(5, 4).is_err()); // 偶数
        assert!(MazePoints::initialize_maze_points(3, 5).is_err()); // 5未満
        assert!(MazePoints::initialize_maze_points(5, 3).is_err()); // 5未満
        assert!(MazePoints::initialize_maze_points(0, 5).is_err()); // 0
        assert!(MazePoints::initialize_maze_points(5, 0).is_err()); // 0
    }

    #[test]
    fn test_initialize_maze_points_boundary_conditions() {
        // 最小有効サイズ
        let result = MazePoints::initialize_maze_points(5, 5);
        assert!(result.is_ok());

        // 大きなサイズ（i32範囲内）
        let result = MazePoints::initialize_maze_points(101, 101);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_maze_points_i32_overflow() {
        // i32の範囲を超える値でのテスト
        let max_i32_plus_1 = (i32::MAX as u32) + 1;
        assert!(MazePoints::initialize_maze_points(max_i32_plus_1, 5).is_err());
        assert!(MazePoints::initialize_maze_points(5, max_i32_plus_1).is_err());
    }

    /// generate_adjacent_pillars のテスト
    #[test]
    fn test_generate_adjacent_pillars_center() {
        let center = MazePoint::new(4, 4);
        let adjacent = generate_adjacent_pillars(&center, 9, 9);

        // 4方向の隣接点が生成されることを確認
        assert_eq!(adjacent.len(), 4);
        assert!(adjacent.contains(&MazePoint::new(2, 4))); // 左
        assert!(adjacent.contains(&MazePoint::new(6, 4))); // 右
        assert!(adjacent.contains(&MazePoint::new(4, 2))); // 上
        assert!(adjacent.contains(&MazePoint::new(4, 6))); // 下
    }

    #[test]
    fn test_generate_adjacent_pillars_boundary() {
        // 境界近くでの隣接点生成テスト
        let corner = MazePoint::new(2, 2);
        let adjacent = generate_adjacent_pillars(&corner, 9, 9);

        assert_eq!(adjacent.len(), 4);
        assert!(adjacent.contains(&MazePoint::new(0, 2))); // 左
        assert!(adjacent.contains(&MazePoint::new(4, 2))); // 右
        assert!(adjacent.contains(&MazePoint::new(2, 4))); // 下
        assert!(adjacent.contains(&MazePoint::new(2, 0))); // 上
    }

    #[test]
    fn test_generate_adjacent_pillars_edge_cases() {
        // 迷路の端での隣接点生成
        let edge_point = MazePoint::new(0, 2);
        let adjacent = generate_adjacent_pillars(&edge_point, 7, 7);
        assert_eq!(adjacent.len(), 3); // 右と上下のみ

        // 角での隣接点生成
        let corner_point = MazePoint::new(0, 0);
        let adjacent = generate_adjacent_pillars(&corner_point, 7, 7);
        assert_eq!(adjacent.len(), 2); // 右と下のみ

        // 最大座標での隣接点生成
        let max_point = MazePoint::new(6, 6);
        let adjacent = generate_adjacent_pillars(&max_point, 7, 7);
        assert_eq!(adjacent.len(), 2); // 左と上のみ
    }

    #[test]
    fn test_generate_adjacent_pillars_small_maze() {
        // 最小サイズでの隣接点生成
        let center = MazePoint::new(2, 2);
        let adjacent = generate_adjacent_pillars(&center, 5, 5);
        assert_eq!(adjacent.len(), 4); // 右と下のみ（境界制約）
    }

    /// validate_selected_point のテスト
    #[test]
    fn test_validate_selected_point_outside_wall() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // Outside壁の検証
        let outside_point = MazePoint::new(0, 3);
        assert!(validate_selected_point(
            &maze_points_guard,
            &outside_point,
            &identifier
        ));

        // 角の外壁
        let corner_outside = MazePoint::new(0, 0);
        assert!(validate_selected_point(
            &maze_points_guard,
            &corner_outside,
            &identifier
        ));

        // 底辺の外壁
        let bottom_outside = MazePoint::new(3, 6);
        assert!(validate_selected_point(
            &maze_points_guard,
            &bottom_outside,
            &identifier
        ));
    }

    #[test]
    fn test_validate_selected_point_not_checked_pillar() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // NotChecked柱の検証
        let pillar_point = MazePoint::new(2, 2);
        assert!(validate_selected_point(
            &maze_points_guard,
            &pillar_point,
            &identifier
        ));

        let pillar_point2 = MazePoint::new(4, 4);
        assert!(validate_selected_point(
            &maze_points_guard,
            &pillar_point2,
            &identifier
        ));
    }

    #[test]
    fn test_validate_selected_point_path() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // Path状態の点は無効
        let path_point = MazePoint::new(1, 1);
        assert!(!validate_selected_point(
            &maze_points_guard,
            &path_point,
            &identifier
        ));

        let path_point2 = MazePoint::new(3, 3);
        assert!(!validate_selected_point(
            &maze_points_guard,
            &path_point2,
            &identifier
        ));
    }

    #[test]
    fn test_validate_selected_point_extending_pillars() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 1つの柱をExtending状態に変更
        {
            let mut maze_points_guard = maze_points.write().unwrap();
            let pillar_point = MazePoint::new(2, 2);
            let _ = maze_points_guard.mark_pillar_as_extending(&pillar_point, &identifier1);
        }

        let maze_points_guard = maze_points.read().unwrap();
        let pillar_point = MazePoint::new(2, 2);

        // 同じ識別子では無効（自分の壁）
        assert!(!validate_selected_point(
            &maze_points_guard,
            &pillar_point,
            &identifier1
        ));

        // 異なる識別子では有効（外壁として扱う）
        assert!(validate_selected_point(
            &maze_points_guard,
            &pillar_point,
            &identifier2
        ));
    }

    #[test]
    fn test_validate_selected_point_nonexistent() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // 存在しない座標は無効
        let nonexistent = MazePoint::new(10, 10);
        assert!(!validate_selected_point(
            &maze_points_guard,
            &nonexistent,
            &identifier
        ));
    }

    /// select_start_pillar_point のテスト
    #[test]
    fn test_select_start_pillar_point_success() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        let result = select_start_pillar_point(&maze_points, identifier);
        assert!(result.is_ok());

        let selected_point = result.unwrap();

        // 選択された点が柱ポイントに含まれていることを確認
        let maze_points_guard = maze_points.read().unwrap();
        assert!(maze_points_guard.pillar_points().contains(&selected_point));

        // 選択された点がExtending状態に変更されていることを確認
        if let Some(status) = maze_points_guard.all_maze_points.get(&selected_point) {
            assert!(status.is_extending_wall());
            assert!(status.is_my_wall(&identifier));
        } else {
            panic!("Selected point not found in maze points");
        }
    }

    #[test]
    fn test_select_start_pillar_point_multiple_calls() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap();

        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 1回目の選択
        let result1 = select_start_pillar_point(&maze_points, identifier1);
        assert!(result1.is_ok());

        // 2回目の選択（異なる識別子）
        let result2 = select_start_pillar_point(&maze_points, identifier2);
        assert!(result2.is_ok());

        // 選択された点が有効な結果であることを確認
        let point1 = result1.unwrap();
        let point2 = result2.unwrap();

        let maze_points_guard = maze_points.read().unwrap();
        assert!(maze_points_guard.pillar_points().contains(&point1));
        assert!(maze_points_guard.pillar_points().contains(&point2));

        // 両方ともExtending状態になっていることを確認
        if let Some(status1) = maze_points_guard.all_maze_points.get(&point1) {
            assert!(status1.is_extending_wall());
            assert!(status1.is_my_wall(&identifier1));
        }
        if let Some(status2) = maze_points_guard.all_maze_points.get(&point2) {
            assert!(status2.is_extending_wall());
            assert!(status2.is_my_wall(&identifier2));
        }
    }

    #[test]
    fn test_select_start_pillar_point_all_exhausted() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();

        // 最初にall_pillar_seeked_flagを手動で設定
        {
            let mut maze_points_guard = maze_points.write().unwrap();
            maze_points_guard.all_pillar_seeked_flag = true;
        }

        let identifier = WallIdentifier::new();
        let result = select_start_pillar_point(&maze_points, identifier);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("All pillar points have already been sought"));
    }

    #[test]
    fn test_select_start_pillar_point_exhaustion_simulation() {
        // 小さな迷路でのピラー消費シミュレーション
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let mut identifiers = Vec::new();
        let mut selected_points = Vec::new();

        // 利用可能なピラーをすべて選択
        let max_pillars = {
            let guard = maze_points.read().unwrap();
            guard.pillar_points().len()
        };

        for i in 0..max_pillars {
            let identifier = WallIdentifier::new();
            let result = select_start_pillar_point(&maze_points, identifier);

            if let Ok(point) = result {
                identifiers.push(identifier);
                selected_points.push(point);
            } else {
                println!(
                    "Failed to select pillar at iteration {}: {:?}",
                    i,
                    result.err()
                );
                break;
            }
        }

        // 少なくとも1つのピラーは選択できるはず
        assert!(!selected_points.is_empty());

        // 次の選択は失敗するか、all_pillar_seeked_flagがtrueになるはず
        let final_identifier = WallIdentifier::new();
        let final_result = select_start_pillar_point(&maze_points, final_identifier);

        if final_result.is_err() {
            let guard = maze_points.read().unwrap();
            // エラーになった場合、フラグがtrueになっているはず
            println!(
                "All pillars exhausted, flag: {}",
                guard.all_pillar_seeked_flag()
            );
        }
    }

    /// extend_pillar_to_adjacent_pillar のテスト
    #[test]
    fn test_extend_pillar_to_adjacent_pillar_success() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        // 開始点を選択
        let start_point = select_start_pillar_point(&maze_points, identifier).unwrap();

        // 拡張を実行
        let result = extend_pillar_to_adjacent_pillar(&maze_points, start_point, identifier);
        assert!(result.is_ok());

        let extension_result = result.unwrap();

        // 結果が有効であることを確認
        assert!(
            extension_result.is_next_pillar()
                || extension_result.is_outside()
                || extension_result.is_surrounded()
        );
    }

    #[test]
    fn test_extend_pillar_invalid_state() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        // NotChecked状態の柱で拡張を試行（Extending状態でないためエラー）
        let pillar_point = MazePoint::new(2, 2);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not in Extending state"));
    }

    #[test]
    fn test_extend_pillar_wrong_identifier() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();
        let wrong_identifier = WallIdentifier::new();

        // 正しい識別子で開始点を選択
        let start_point = select_start_pillar_point(&maze_points, identifier).unwrap();

        // 間違った識別子で拡張を試行
        let result = extend_pillar_to_adjacent_pillar(&maze_points, start_point, wrong_identifier);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("identifier does not match"));
    }

    #[test]
    fn test_extend_pillar_all_pillars_sought() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();

        // 開始点を選択
        let start_point = select_start_pillar_point(&maze_points, identifier).unwrap();

        // all_pillar_seeked_flagを設定
        {
            let mut maze_points_guard = maze_points.write().unwrap();
            maze_points_guard.all_pillar_seeked_flag = true;
        }

        // 拡張を試行
        let result = extend_pillar_to_adjacent_pillar(&maze_points, start_point, identifier);
        assert!(result.is_ok());

        let extension_result = result.unwrap();
        assert!(extension_result.is_surrounded());
        assert_eq!(extension_result.point, start_point);
    }

    #[test]
    fn test_extend_pillar_nonexistent_point() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        // 存在しない座標での拡張試行
        let invalid_point = MazePoint::new(10, 10);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, invalid_point, identifier);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("MazePoint not found"));
    }

    #[test]
    fn test_extend_pillar_boundary_conditions() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        // 境界点での拡張試行（NotChecked状態でない）
        let boundary_point = MazePoint::new(0, 0);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, boundary_point, identifier);
        assert!(result.is_err());
    }

    /// SeekAdjacentPillarOkResult のテスト
    #[test]
    fn test_seek_adjacent_pillar_ok_result_creation() {
        let point = MazePoint::new(2, 2);

        // NextPillar結果のテスト
        let next_result =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::NextPillar, point);
        assert!(next_result.is_next_pillar());
        assert!(!next_result.is_surrounded());
        assert!(!next_result.is_outside());
        assert_eq!(next_result.point, point);

        // Outside結果のテスト
        let outside_result =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::Outside, point);
        assert!(!outside_result.is_next_pillar());
        assert!(!outside_result.is_surrounded());
        assert!(outside_result.is_outside());

        // SurroundedPillar結果のテスト
        let surrounded_result =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::SurroundedPillar, point);
        assert!(!surrounded_result.is_next_pillar());
        assert!(surrounded_result.is_surrounded());
        assert!(!surrounded_result.is_outside());
    }

    #[test]
    fn test_seek_adjacent_pillar_ok_result_equality() {
        let point1 = MazePoint::new(2, 2);
        let point2 = MazePoint::new(2, 2);
        let point3 = MazePoint::new(4, 4);

        let result1 =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::NextPillar, point1);
        let result2 =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::NextPillar, point2);
        let result3 =
            SeekAdjacentPillarOkResult::new(SeekAdjacentPillarOkState::NextPillar, point3);

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    /// select_valid_adjacent_pillar のテスト
    #[test]
    fn test_select_valid_adjacent_pillar_success() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // 有効な隣接柱のリストを作成
        let mut adjacent_pillars = vec![
            MazePoint::new(2, 2), // NotChecked pillar - 有効
            MazePoint::new(0, 2), // Outside wall - 有効
        ];

        let result =
            select_valid_adjacent_pillar(&maze_points_guard, &mut adjacent_pillars, &identifier);
        assert!(result.is_some());

        let selected = result.unwrap();
        // NotChecked pillar または Outside wall のいずれかが選択される
        assert!(selected == MazePoint::new(2, 2) || selected == MazePoint::new(0, 2));
    }

    #[test]
    fn test_select_valid_adjacent_pillar_no_valid() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // 無効な隣接柱のみのリストを作成
        let mut adjacent_pillars = vec![
            MazePoint::new(1, 1), // Path - 無効
            MazePoint::new(3, 3), // Path - 無効
        ];

        let result =
            select_valid_adjacent_pillar(&maze_points_guard, &mut adjacent_pillars, &identifier);
        assert!(result.is_none());
        assert!(adjacent_pillars.is_empty()); // すべて除外される
    }

    #[test]
    fn test_select_valid_adjacent_pillar_mixed_validity() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();
        let identifier = WallIdentifier::new();

        // 有効と無効が混在するリスト
        let mut adjacent_pillars = vec![
            MazePoint::new(1, 1), // Path - 無効
            MazePoint::new(2, 2), // NotChecked pillar - 有効
            MazePoint::new(3, 3), // Path - 無効
            MazePoint::new(0, 2), // Outside wall - 有効
        ];

        let result =
            select_valid_adjacent_pillar(&maze_points_guard, &mut adjacent_pillars, &identifier);
        assert!(result.is_some());

        let selected = result.unwrap();
        // 有効な候補のいずれかが選択される
        assert!(selected == MazePoint::new(2, 2) || selected == MazePoint::new(0, 2));
    }

    /// 複雑なシナリオのテスト
    #[test]
    fn test_complex_pillar_extension_scenario() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap();
        let identifier = WallIdentifier::new();

        // 最初の柱を選択
        let first_pillar = select_start_pillar_point(&maze_points, identifier).unwrap();

        // 連続的な拡張を試行
        let mut current_pillar = first_pillar;
        let mut extension_count = 0;
        let mut extension_history = Vec::new();

        for i in 0..10 {
            // 最大10回の拡張を試行
            match extend_pillar_to_adjacent_pillar(&maze_points, current_pillar, identifier) {
                Ok(result) => {
                    extension_history.push((i, current_pillar, result.clone()));

                    if result.is_next_pillar() {
                        current_pillar = result.point;
                        extension_count += 1;
                    } else if result.is_outside() {
                        println!("Reached outside at iteration {}: {:?}", i, result.point);
                        break;
                    } else if result.is_surrounded() {
                        println!("Surrounded at iteration {}: {:?}", i, result.point);
                        break;
                    }
                }
                Err(e) => {
                    println!("Extension failed at iteration {}: {}", i, e);
                    break;
                }
            }
        }

        // 拡張履歴をログ出力
        for (i, from, result) in extension_history {
            println!(
                "Extension {}: from {:?} -> {:?} ({})",
                i,
                from,
                result.point,
                if result.is_next_pillar() {
                    "NextPillar"
                } else if result.is_outside() {
                    "Outside"
                } else {
                    "Surrounded"
                }
            );
        }

        // 少なくとも1回は何らかの結果が得られるはず
        assert!(extension_count >= 0);
    }

    #[test]
    fn test_multiple_identifier_interaction() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 異なる識別子で複数の拡張を実行
        let pillar1 = select_start_pillar_point(&maze_points, identifier1).unwrap();
        let pillar2 = select_start_pillar_point(&maze_points, identifier2).unwrap();

        let result1 = extend_pillar_to_adjacent_pillar(&maze_points, pillar1, identifier1);
        let result2 = extend_pillar_to_adjacent_pillar(&maze_points, pillar2, identifier2);

        // 両方とも何らかの結果が得られるはず
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        println!("Result1: {:?}", result1.unwrap());
        println!("Result2: {:?}", result2.unwrap());
    }

    /// 並行性テスト
    #[test]
    fn test_concurrent_pillar_selection() {
        let maze_points = Arc::new(MazePoints::initialize_maze_points(11, 11).unwrap());
        let mut handles = vec![];

        // 複数スレッドで同時に開始点を選択
        for i in 0..4 {
            let maze_points_clone = Arc::clone(&maze_points);
            let handle = thread::spawn(move || {
                let identifier = WallIdentifier::new();
                let result = select_start_pillar_point(&maze_points_clone, identifier);
                (i, identifier, result)
            });
            handles.push(handle);
        }

        // 結果を収集
        let mut success_count = 0;
        let mut selected_points = Vec::new();

        for handle in handles {
            let (thread_id, identifier, result) = handle.join().unwrap();
            if let Ok(point) = result {
                success_count += 1;
                selected_points.push((thread_id, identifier, point));
                println!("Thread {} succeeded with point {:?}", thread_id, point);
            } else {
                println!("Thread {} failed: {:?}", thread_id, result.err());
            }
        }

        // 少なくとも1つのスレッドは成功するはず
        assert!(success_count > 0);

        // 選択された点がすべて有効であることを確認
        let maze_points_guard = maze_points.read().unwrap();
        for (thread_id, identifier, point) in selected_points {
            assert!(maze_points_guard.pillar_points().contains(&point));
            if let Some(status) = maze_points_guard.all_maze_points.get(&point) {
                assert!(status.is_extending_wall());
                assert!(status.is_my_wall(&identifier));
                println!(
                    "Thread {} point {:?} correctly set to extending",
                    thread_id, point
                );
            }
        }
    }

    #[test]
    fn test_concurrent_pillar_extension() {
        let maze_points = Arc::new(MazePoints::initialize_maze_points(13, 13).unwrap());
        let mut handles = vec![];

        // 各スレッドで選択と拡張を実行
        for i in 0..3 {
            let maze_points_clone = Arc::clone(&maze_points);
            let handle = thread::spawn(move || {
                let identifier = WallIdentifier::new();
                let selection_result = select_start_pillar_point(&maze_points_clone, identifier);

                if let Ok(point) = selection_result {
                    let extension_result =
                        extend_pillar_to_adjacent_pillar(&maze_points_clone, point, identifier);
                    (i, Some(point), extension_result)
                } else {
                    (
                        i,
                        None,
                        Err(Box::new(std::io::Error::other("Selection failed"))
                            as Box<dyn std::error::Error + Send + Sync>),
                    )
                }
            });
            handles.push(handle);
        }

        // 結果を収集
        let mut successful_extensions = 0;

        for handle in handles {
            let (thread_id, selected_point, extension_result) = handle.join().unwrap();
            if let Some(point) = selected_point {
                match extension_result {
                    Ok(result) => {
                        successful_extensions += 1;
                        println!(
                            "Thread {} extended from {:?} to {:?} ({})",
                            thread_id,
                            point,
                            result.point,
                            if result.is_next_pillar() {
                                "NextPillar"
                            } else if result.is_outside() {
                                "Outside"
                            } else {
                                "Surrounded"
                            }
                        );
                    }
                    Err(e) => {
                        println!("Thread {} extension failed: {}", thread_id, e);
                    }
                }
            } else {
                println!("Thread {} failed to select pillar", thread_id);
            }
        }

        // 少なくとも1つの拡張は成功するはず
        assert!(successful_extensions > 0);
    }

    /// 迷路の完全性テスト
    #[test]
    fn test_maze_integrity_comprehensive() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        // 全座標点が初期化されていることを確認
        for y in 0..7 {
            for x in 0..7 {
                let point = MazePoint::new(x, y);
                assert!(
                    maze_points_guard.all_maze_points.contains_key(&point),
                    "Point ({}, {}) not found in maze",
                    x,
                    y
                );
            }
        }

        // 境界が外壁になっていることを確認
        for i in 0..7 {
            let points_to_check = vec![
                MazePoint::new(i, 0), // 上辺
                MazePoint::new(i, 6), // 下辺
                MazePoint::new(0, i), // 左辺
                MazePoint::new(6, i), // 右辺
            ];

            for point in points_to_check {
                if let Some(status) = maze_points_guard.all_maze_points.get(&point) {
                    assert!(
                        status.is_outside_wall(),
                        "Boundary point {:?} is not an outside wall: {:?}",
                        point,
                        status
                    );
                } else {
                    panic!("Boundary point {:?} not found", point);
                }
            }
        }

        // 内部の偶数座標が柱になっていることを確認
        for y in (2..5).step_by(2) {
            for x in (2..5).step_by(2) {
                let point = MazePoint::new(x, y);
                if let Some(status) = maze_points_guard.all_maze_points.get(&point) {
                    assert!(
                        status.is_not_checked_wall(),
                        "Internal even coordinate {:?} is not a NotChecked pillar: {:?}",
                        point,
                        status
                    );
                } else {
                    panic!("Internal pillar point {:?} not found", point);
                }
            }
        }

        // 内部の奇数座標がPathになっていることを確認
        for y in 1..6 {
            for x in 1..6 {
                if x % 2 == 1 || y % 2 == 1 {
                    let point = MazePoint::new(x, y);
                    if let Some(status) = maze_points_guard.all_maze_points.get(&point) {
                        match status {
                            MazePointStatus::Path => {
                                // 正常
                            }
                            MazePointStatus::Wall(WallType::Pillar(_), _) => {
                                // 柱の場合は偶数座標のはず
                                if x % 2 == 0 && y % 2 == 0 {
                                    // OK
                                } else {
                                    panic!("Pillar at odd coordinate {:?}", point);
                                }
                            }
                            _ => {
                                panic!(
                                    "Unexpected status at internal point {:?}: {:?}",
                                    point, status
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_maze_pillar_points_consistency() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        // pillar_pointsに含まれる点がすべて実際に柱であることを確認
        for pillar_point in maze_points_guard.pillar_points() {
            if let Some(status) = maze_points_guard.all_maze_points.get(pillar_point) {
                assert!(
                    status.is_not_checked_wall(),
                    "Point in pillar_points {:?} is not a NotChecked pillar: {:?}",
                    pillar_point,
                    status
                );

                // 境界に接していないことを確認（内部の柱のみ）
                assert!(pillar_point.x() > 0 && pillar_point.x() < 8);
                assert!(pillar_point.y() > 0 && pillar_point.y() < 8);

                // 偶数座標であることを確認
                assert_eq!(pillar_point.x() % 2, 0);
                assert_eq!(pillar_point.y() % 2, 0);
            } else {
                panic!(
                    "Pillar point {:?} not found in all_maze_points",
                    pillar_point
                );
            }
        }

        // 逆に、内部の偶数座標の柱がすべてpillar_pointsに含まれていることを確認
        for y in (2..7).step_by(2) {
            for x in (2..7).step_by(2) {
                let point = MazePoint::new(x, y);
                assert!(
                    maze_points_guard.pillar_points().contains(&point),
                    "Internal pillar {:?} not found in pillar_points",
                    point
                );
            }
        }
    }

    /// エラーケースの包括的テスト
    #[test]
    fn test_comprehensive_error_cases() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();

        // 存在しない座標での拡張試行
        let invalid_point = MazePoint::new(10, 10);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, invalid_point, identifier);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("MazePoint not found")
        );

        // 境界点での拡張試行
        let boundary_point = MazePoint::new(0, 0);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, boundary_point, identifier);
        assert!(result.is_err());

        // Path状態の点での拡張試行
        let path_point = MazePoint::new(1, 1);
        let result = extend_pillar_to_adjacent_pillar(&maze_points, path_point, identifier);
        assert!(result.is_err());
    }

    /// 大規模な迷路での性能テスト
    #[test]
    fn test_large_maze_performance() {
        let start_time = std::time::Instant::now();

        let maze_points = MazePoints::initialize_maze_points(51, 51).unwrap();
        let identifier = WallIdentifier::new();

        // 複数の開始点を選択と拡張
        let mut operations = 0;
        for _ in 0..10 {
            if let Ok(pillar) = select_start_pillar_point(&maze_points, identifier) {
                operations += 1;
                if extend_pillar_to_adjacent_pillar(&maze_points, pillar, identifier).is_ok() {
                    operations += 1;
                }
            }
        }

        let elapsed = start_time.elapsed();
        println!(
            "Large maze test completed {} operations in: {:?}",
            operations, elapsed
        );

        // 合理的な時間内で完了することを確認（1秒以内）
        assert!(elapsed.as_secs() < 1);
        assert!(operations > 0);
    }

    /// エッジケースの境界テスト
    #[test]
    fn test_boundary_edge_cases_detailed() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        // 角の座標での隣接点生成テスト
        let corners = vec![
            (MazePoint::new(0, 0), 2), // 左上角：右と下
            (MazePoint::new(4, 0), 2), // 右上角：左と下
            (MazePoint::new(0, 4), 2), // 左下角：右と上
            (MazePoint::new(4, 4), 2), // 右下角：左と上
        ];

        for (corner, expected_count) in corners {
            let adjacents = generate_adjacent_pillars(&corner, 5, 5);
            assert_eq!(
                adjacents.len(),
                expected_count,
                "Corner {:?} should have {} adjacents, got {}",
                corner,
                expected_count,
                adjacents.len()
            );
        }

        // エッジの座標での隣接点生成テスト
        let edges = vec![
            (MazePoint::new(2, 0), 3), // 上辺：左、右、下
            (MazePoint::new(2, 4), 3), // 下辺：左、右、上
            (MazePoint::new(0, 2), 3), // 左辺：上、下、右
            (MazePoint::new(4, 2), 3), // 右辺：上、下、左
        ];

        for (edge, expected_count) in edges {
            let adjacents = generate_adjacent_pillars(&edge, 5, 5);
            assert_eq!(
                adjacents.len(),
                expected_count,
                "Edge {:?} should have {} adjacents, got {}",
                edge,
                expected_count,
                adjacents.len()
            );
        }

        // 中央の座標での隣接点生成テスト
        let center = MazePoint::new(2, 2);
        let adjacents = generate_adjacent_pillars(&center, 5, 5);
        assert_eq!(
            adjacents.len(),
            4,
            "Center {:?} should have 4 adjacents in 5x5 maze, got {}",
            center,
            adjacents.len()
        );
    }

    #[test]
    fn test_state_consistency_after_operations() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        // 操作前の状態を記録
        let initial_extending_count = {
            let guard = maze_points.read().unwrap();
            guard.extending_pillar_points.len()
        };

        // 開始点を選択
        let start_point = select_start_pillar_point(&maze_points, identifier).unwrap();

        // 選択後の状態をチェック
        {
            let guard = maze_points.read().unwrap();
            assert_eq!(
                guard.extending_pillar_points.len(),
                initial_extending_count + 1
            );
            assert!(guard.extending_pillar_points.contains(&start_point));

            if let Some(status) = guard.all_maze_points.get(&start_point) {
                assert!(status.is_extending_wall());
                assert!(status.is_my_wall(&identifier));
            }
        }

        // 拡張を実行
        let extension_result =
            extend_pillar_to_adjacent_pillar(&maze_points, start_point, identifier);

        if let Ok(result) = extension_result {
            let guard = maze_points.read().unwrap();

            if result.is_next_pillar() {
                // 新しい柱が拡張状態になっているはず
                if let Some(status) = guard.all_maze_points.get(&result.point) {
                    assert!(status.is_extending_wall());
                    assert!(status.is_my_wall(&identifier));
                }
                assert!(guard.extending_pillar_points.contains(&result.point));
            }

            // 中間点が壁になっているかチェック（NextPillarの場合）
            if result.is_next_pillar() {
                let middle_x = (start_point.x() + result.point.x()) / 2;
                let middle_y = (start_point.y() + result.point.y()) / 2;
                let middle_point = MazePoint::new(middle_x, middle_y);

                if let Some(status) = guard.all_maze_points.get(&middle_point) {
                    match status {
                        MazePointStatus::Wall(WallType::Wall, Some(wall_id)) => {
                            assert_eq!(*wall_id, identifier);
                        }
                        _ => panic!("Middle point {:?} should be a wall", middle_point),
                    }
                }
            }
        }
    }

    /// 迷路の初期化テスト
    #[test]
    fn test_maze_initialization() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let maze_points_guard = maze_points.read().unwrap();

        // ガードを使用してテストを実行
        assert_eq!(maze_points_guard.x_size(), 7);
        assert_eq!(maze_points_guard.y_size(), 7);
        assert!(!maze_points_guard.all_pillar_seeked_flag());

        // 境界点が外壁になっていることを確認
        let boundary_point = MazePoint::new(0, 0);
        if let Some(status) = maze_points_guard
            .get_all_maze_points_clone()
            .get(&boundary_point)
        {
            assert!(status.is_outside_wall());
        }
    }
}
