use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{
    maze_point::MazePoint,
    maze_point_status::{MazePointStatus, PillarExtendStatus, WallIdentifier, WallType},
};
use rand::Rng;

/// 迷路の構成要素とその状態を管理する構造体
///
/// この構造体は迷路の各座標点の状態（道、壁、柱）を管理し、
/// 迷路生成アルゴリズムの進行状況を追跡します。
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
    /// 拡張処理済みの柱の座標集合
    extended_pillar_points: HashSet<MazePoint>,
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
        self.extended_pillar_points.clone()
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
        //引数がi32の範囲内の値であることを確認する。
        if x_size > i32::MAX as u32 || y_size > i32::MAX as u32 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "x_size and y_size must be within the range of i32",
            )));
        }
        let mut all_maze_points: HashMap<MazePoint, MazePointStatus> = HashMap::new();
        for y in 0..y_size {
            for x in 0..x_size {
                let point = MazePoint::new(x, y);
                all_maze_points.insert(point, MazePointStatus::Path);
            }
        }
        let mut wall_start_points = HashSet::new();
        let id = WallIdentifier::new();
        //x=0もしくは x=x_size-1のときは壁
        //y=0もしくは y=y_size-1のときは壁
        for y in 0..y_size {
            for x in 0..x_size {
                if x == 0 || x == x_size - 1 || y == 0 || y == y_size - 1 {
                    let point = MazePoint::new(x, y);
                    if let Some(point_value) = all_maze_points.get_mut(&point) {
                        *point_value = MazePointStatus::new_outside_wall(id.clone());
                    }
                }
            }
        }
        //xが偶数かつyが偶数のときは柱(すでに壁の部分は除く)
        for y in 0..y_size {
            for x in 0..x_size {
                if x % 2 == 0 && y % 2 == 0 {
                    let point = MazePoint::new(x, y);
                    if let Some(point_value) = all_maze_points.get_mut(&point)
                        && point_value == &MazePointStatus::Path
                    {
                        // すでにPathであればPillarに変更
                        *point_value = MazePointStatus::new_notchecked_pillar();
                        if x > 0 && y > 0 && x < x_size - 1 && y < y_size - 1 {
                            wall_start_points.insert(point.clone());
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
            extended_pillar_points: HashSet::new(),
        })))
    }

    /// 柱を拡張済み状態に変更し、拡張済み柱リストに追加する
    ///
    /// # Arguments
    ///
    /// * `pillar_point` - 拡張済みにする柱の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Note
    ///
    /// この関数は書き込みロックが取得済みの状態で呼び出される必要があります
    fn mark_pillar_as_extended(&mut self, pillar_point: &MazePoint, identifier: &WallIdentifier) {
        self.all_maze_points.insert(
            pillar_point.clone(),
            MazePointStatus::Wall(
                WallType::Pillar(PillarExtendStatus::Extended),
                Some(identifier.clone()),
            ),
        );
        self.extended_pillar_points.insert(pillar_point.clone());
    }

    /// 柱を進行中状態に変更する
    ///
    /// # Arguments
    ///
    /// * `pillar_point` - 進行中にする柱の座標
    /// * `identifier` - 壁の識別子
    fn mark_pillar_as_in_progress(
        &mut self,
        pillar_point: &MazePoint,
        identifier: &WallIdentifier,
    ) {
        self.all_maze_points.insert(
            pillar_point.clone(),
            MazePointStatus::Wall(
                WallType::Pillar(PillarExtendStatus::InProgress),
                Some(identifier.clone()),
            ),
        );
    }

    /// 中間点を壁に変更する
    ///
    /// # Arguments
    ///
    /// * `middle_point` - 壁にする中間点の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(())、中間点がPathでない場合はエラー
    fn mark_middle_point_as_wall(
        &mut self,
        middle_point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(status) = self.all_maze_points.get(middle_point) {
            match status {
                MazePointStatus::Path => {
                    self.all_maze_points.insert(
                        middle_point.clone(),
                        MazePointStatus::Wall(WallType::Wall, Some(identifier.clone())),
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
    /// # Arguments
    ///
    /// * `from_pillar` - 拡張元の柱
    /// * `to_pillar` - 拡張先の柱
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合は拡張結果、失敗した場合はエラー
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

        // 元の柱を拡張済みに変更
        self.mark_pillar_as_extended(from_pillar, identifier);

        // 拡張先の状態を確認して適切に処理
        if let Some(status) = self.all_maze_points.get(to_pillar).cloned() {
            match status {
                // Outside壁の場合
                MazePointStatus::Wall(WallType::Outside, _) => Ok(SeekAdjacentPillarOkResult::new(
                    SeekAdjacentPillarOkState::Outside,
                    to_pillar.clone(),
                )),
                // NotChecked状態の柱の場合
                MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), _) => {
                    // 拡張先の柱を進行中に変更
                    self.mark_pillar_as_in_progress(to_pillar, identifier);
                    Ok(SeekAdjacentPillarOkResult::new(
                        SeekAdjacentPillarOkState::NextPillar,
                        to_pillar.clone(),
                    ))
                }
                _ => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unexpected pillar state at {:?}: {:?}", to_pillar, status),
                ))),
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
                    if self.extended_pillar_points.contains(*point) {
                        return false;
                    }

                    // 状態がNotCheckedの柱のみ選択対象
                    if let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), _)) =
                        self.all_maze_points.get(point)
                    {
                        *extend_status == PillarExtendStatus::NotChecked
                    } else {
                        false
                    }
                })
                .cloned()
                .collect()
        };

        (self.all_pillar_seeked_flag, candidates)
    }

    /// 指定された柱をInProgress状態に変更できるかチェックし、可能であれば変更する
    ///
    /// # Arguments
    ///
    /// * `point` - 対象の柱の座標
    /// * `identifier` - 壁の識別子
    ///
    /// # Returns
    ///
    /// 成功した場合はOk(true)、状態が変更されていた場合はOk(false)、エラーの場合はErr
    fn try_mark_pillar_as_in_progress(
        &mut self,
        point: &MazePoint,
        identifier: &WallIdentifier,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // 拡張済みでなく、NotChecked状態であることを確認
        if !self.extended_pillar_points.contains(point)
            && let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), _)) =
                self.all_maze_points.get(point)
            && *extend_status == PillarExtendStatus::NotChecked
        {
            self.all_maze_points.insert(
                point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );
            return Ok(true);
        }
        Ok(false)
    }
}

/// 迷路生成の開始点となる柱をランダムに選択する
///
/// この関数は利用可能な柱（NotChecked状態かつ拡張済みでない）の中からランダムに1つを選択し、
/// InProgress状態に変更してから返します。スレッドセーフな操作を行い、
/// 複数のスレッドから同時にアクセスされても安全です。
/// 利用可能な柱がない場合、all_pillar_seeked_flagをtrueに設定します。
///
/// **最適化点**: 読み取りロックで候補の選択を行い、書き込みロックは状態変更のみに使用します。
/// **変更点**: `extended_pillar_points`メンバを使用して拡張済み柱を効率的に除外します。
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
        let selected_point = candidates[random_index].clone();

        // フェーズ3: 書き込みロックで状態変更を試行
        {
            let mut write_guard = maze_points.write().map_err(|_| {
                Box::new(std::io::Error::other(format!(
                    "Failed to acquire write lock. {}",
                    error_msg_common_part
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            match write_guard.try_mark_pillar_as_in_progress(&selected_point, &identifier)? {
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
/// この関数は指定された柱（InProgress状態）から隣接する利用可能な柱への拡張を試行します。
/// 成功した場合、元の柱はExtended状態になり、選択された隣接柱はInProgress状態になります。
/// また、両柱の間の座標点はWall状態に変更されます。
///
/// 隣接する柱の候補は距離2の位置（上下左右）にある座標点で、以下の条件を満たすもの：
/// - NotChecked状態の柱
/// - Outside壁（境界）
/// - InProgressまたはExtended状態の柱で、識別子が異なる場合（外壁として扱う）
///
/// 中間点（両柱の中点）はPath状態である必要があり、拡張時にWall状態に変更されます。
/// 利用可能な隣接柱がない場合は、元の柱をExtended状態に変更してSurroundedPillar結果を返します。
///
/// **重要な変更点**:
/// - `all_pillar_seeked_flag`がtrueの場合、エラーではなく`SurroundedPillar`結果を返します。
/// - InProgressもしくはExtendedのpillarで、identifierが引数と異なる場合は外壁と同じ扱いをします。
/// - 読み取りロックを活用して並行性を向上させます。
/// - unwrap()をエラーハンドリングに置き換えます。
///
/// # Arguments
///
/// * `maze_points` - 迷路データへの参照
/// * `pillar_point` - 拡張元の柱の座標（InProgress状態である必要があります）
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
/// * 指定された柱がInProgress状態でない場合
/// * 識別子が一致しない場合
/// * 中間点がPath状態でない場合
/// * ロックの取得に失敗した場合
/// * 指定された座標点が見つからない場合
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
    let pillar_point_copy = pillar_point.clone();
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
                pillar_point_copy,
            ));
        }

        // 引数として受け取ったMazePointの状態確認
        if let Some(status) = maze_points_read
            .all_maze_points
            .get(&pillar_point_copy)
            .cloned()
        {
            let status_str = format!("from_pillar_status:{:?}", &status);
            if let MazePointStatus::Wall(WallType::Pillar(extend_status), id) = status {
                if extend_status != PillarExtendStatus::InProgress || id != Some(identifier.clone())
                {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Pillar is not in InProgress state or identifier does not match. {} {}",
                            error_msg_common_part, status_str
                        ),
                    )));
                }
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "MazePoint is not a Pillar in InProgress state. status: {} {}",
                        status_str, error_msg_common_part
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
            maze_points_guard.mark_pillar_as_extended(&pillar_point, &identifier);
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
        let selected_point = adjacent_pillars[random_index].clone();

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
    if pillar_point.x() > 2 {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() - 2, pillar_point.y()));
    }
    if pillar_point.x() + 2 < x_size {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() + 2, pillar_point.y()));
    }

    // y方向の隣接点 (x, y±2)
    if pillar_point.y() > 2 {
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
/// 以下の場合を有効とします：
/// - Outside壁（境界）
/// - NotChecked状態の柱
/// - InProgressまたはExtended状態の柱で、識別子が異なる場合（外壁として扱う）
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
        match status {
            // Outside壁は常に有効
            MazePointStatus::Wall(WallType::Outside, _) => true,
            // NotChecked状態の柱は有効
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), _) => true,
            // その他の状態は無効
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze_point_status::{PillarExtendStatus, WallType};

    #[test]
    fn test_select_start_pillar_point_with_extended_pillars() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 最初の柱を選択
        let first_pillar = select_start_pillar_point(&maze_points, identifier1.clone()).unwrap();

        // 最初の柱を拡張済みに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.mark_pillar_as_extended(&first_pillar, &identifier1);
        }

        // 2番目の柱を選択（拡張済みの柱は除外される）
        let second_pillar = select_start_pillar_point(&maze_points, identifier2).unwrap();

        // 選択された柱が異なることを確認
        assert_ne!(first_pillar, second_pillar);

        // 拡張済み柱リストを確認
        let maze_guard = maze_points.read().unwrap();
        assert!(maze_guard.extended_pillar_points.contains(&first_pillar));
        assert!(!maze_guard.extended_pillar_points.contains(&second_pillar));
    }

    #[test]
    fn test_extend_pillar_different_identifier_as_outside() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 最初の柱をInProgressに設定
        let pillar1 = MazePoint::new(2, 2);
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.mark_pillar_as_in_progress(&pillar1, &identifier1);
        }

        // 隣接する柱を異なる識別子でExtendedに設定
        let pillar2 = MazePoint::new(4, 2);
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.mark_pillar_as_extended(&pillar2, &identifier2);
        }

        // pillar1からの拡張を試行
        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar1.clone(), identifier1);

        assert!(result.is_ok());
        let seek_result = result.unwrap();

        // 異なる識別子の柱は外壁として扱われるため、Outsideが返される
        if seek_result.point == pillar2 {
            assert!(
                seek_result.is_outside(),
                "Different identifier pillar should be treated as Outside"
            );
        }

        // pillar1がExtendedになっていることを確認
        let maze_guard = maze_points.read().unwrap();
        if let Some(MazePointStatus::Wall(WallType::Pillar(status), _)) =
            maze_guard.all_maze_points.get(&pillar1)
        {
            assert_eq!(*status, PillarExtendStatus::Extended);
        }
    }

    #[test]
    fn test_validate_selected_point_different_identifiers() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        let pillar_point = MazePoint::new(2, 2);

        {
            let mut maze_guard = maze_points.write().unwrap();
            // pillar_pointを異なる識別子でInProgressに設定
            maze_guard.mark_pillar_as_in_progress(&pillar_point, &identifier2);
        }

        let maze_guard = maze_points.read().unwrap();

        // identifier1での検証では、異なる識別子のため有効と判定される（外壁として扱う）
        assert!(validate_selected_point(
            &maze_guard,
            &pillar_point,
            &identifier1
        ));

        // identifier2での検証では、同じ識別子のため無効と判定される
        assert!(!validate_selected_point(
            &maze_guard,
            &pillar_point,
            &identifier2
        ));
    }

    #[test]
    fn test_generate_adjacent_pillars() {
        let pillar_point = MazePoint::new(4, 4);
        let adjacent_pillars = generate_adjacent_pillars(&pillar_point, 9, 9);

        // 中央の柱からは4方向の隣接柱がある
        assert_eq!(adjacent_pillars.len(), 4);
        assert!(adjacent_pillars.contains(&MazePoint::new(2, 4))); // 左
        assert!(adjacent_pillars.contains(&MazePoint::new(6, 4))); // 右
        assert!(adjacent_pillars.contains(&MazePoint::new(4, 2))); // 上
        assert!(adjacent_pillars.contains(&MazePoint::new(4, 6))); // 下
    }

    #[test]
    fn test_generate_adjacent_pillars_boundary() {
        let pillar_point = MazePoint::new(2, 2);
        let adjacent_pillars = generate_adjacent_pillars(&pillar_point, 7, 7);

        // 境界近くの柱からは2方向の隣接柱がある
        assert_eq!(adjacent_pillars.len(), 2);
        assert!(adjacent_pillars.contains(&MazePoint::new(4, 2))); // 右
        assert!(adjacent_pillars.contains(&MazePoint::new(2, 4))); // 下
    }
}
