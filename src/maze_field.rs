use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    maze_point::MazePoint,
    maze_point_status::{self, MazePointStatus, PillarExtendStatus, WallIdentifier, WallType},
};
use rand::Rng;

#[derive(Debug, Clone)]
pub(crate) struct MazePoints {
    ///迷路のX方向の大きさ。
    x_size: u32,
    ///迷路のY方向の大きさ。
    y_size: u32,
    all_maze_points: HashMap<MazePoint, MazePointStatus>,
    ///全柱探索完了フラグ
    all_pillar_seeked_flag: bool,
    ///迷路の柱(壁の生成起点)の座標
    pillar_points: HashSet<MazePoint>,
}
impl MazePoints {
    /// Returns the x_size of the maze.
    pub fn x_size(&self) -> u32 {
        self.x_size
    }

    /// Returns the y_size of the maze.
    pub fn y_size(&self) -> u32 {
        self.y_size
    }

    /// すべての柱が探索された場合はtrueを返す。
    pub fn all_pillar_seeked_flag(&self) -> bool {
        self.all_pillar_seeked_flag
    }

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
        let id = MazePointStatus::new_identifier();
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
        })))
    }
}

/// 1.&Arc<RwLock<MazePoints>>とWallIdentifierを引数として受け取る。
/// 2.作業用に空のHashSetを作成する。
/// 3.作業用のHashSetの長さがMazePointsのpillar_pointsと同じであるか確認する。同じ場合は全柱探索完了フラグをセットしてエラーを返す。
/// 4.MazePointsのpillar_pointsに格納されている値で、作業用のHashSetに存在しない値から、ランダムで1つMazePointを取得する。
/// 5.作業用のHashSetに取得したMazePointがないことを確認する。ある場合は3からもう1度実施する。
/// 6.取得したMazePointを作業用のHashSetに格納する。
/// 7.取得したMazePointに対応するall_maze_pointsの値がMazePointStatusがWallでWallTypeがPillarで、PillarExtendStatusがNotCheckedであるか確認する。NotCheckedではない場合は3からもう1度実施する。
/// 8.PillarExtendStatusがNotCheckedの場合は、PillarExtendStatusをInProgressに変更する。
/// 9.取得したMazePointを返す。
pub fn select_start_pillar_point(
    maze_points: &Arc<RwLock<MazePoints>>,
    identifier: WallIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(" identifier: {}", identifier.as_str());
    let mut work_set = HashSet::new();

    loop {
        // 書き込みロックを一度だけ取得してアトミックに処理
        let mut maze_points_guard = maze_points.write().map_err(|_| {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire write lock. {}",
                error_msg_common_part
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        if maze_points_guard.all_pillar_seeked_flag {
            return Err(Box::new(std::io::Error::other(format!(
                "All pillar points have already been sought. {}",
                error_msg_common_part
            ))));
        }

        if work_set.len() == maze_points_guard.pillar_points.len() {
            maze_points_guard.all_pillar_seeked_flag = true;
            return Err(Box::new(std::io::Error::other(format!(
                "No more pillar points available. {}",
                error_msg_common_part
            ))));
        }

        // NotCheckedかつ未試行のポイントをフィルタリング
        let candidates: Vec<MazePoint> = maze_points_guard
            .pillar_points
            .iter()
            .filter(|point| {
                if work_set.contains(*point) {
                    return false;
                }
                if let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), _)) =
                    maze_points_guard.all_maze_points.get(point)
                {
                    *extend_status == PillarExtendStatus::NotChecked
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        if candidates.is_empty() {
            return Err(Box::new(std::io::Error::other(format!(
                "No valid pillar points found. {}",
                error_msg_common_part
            ))));
        }

        let mut rng = rand::rng();
        let random_index = rng.random_range(0..candidates.len());
        let point = candidates[random_index].clone();

        // 再度状態を確認（ダブルチェック）
        if let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), _)) =
            maze_points_guard.all_maze_points.get(&point)
            && *extend_status == PillarExtendStatus::NotChecked
        {
            // アトミックに状態を変更
            maze_points_guard.all_maze_points.insert(
                point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier),
                ),
            );
            return Ok(point);
        }

        // 状態が変わっていた場合は作業セットに追加して再試行
        work_set.insert(point);
        // ロックは自動的に解放されて次のループで再取得
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SeekAdjacentPillarOkState {
    NextPillar,
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeekAdjacentPillarOkResult {
    state: SeekAdjacentPillarOkState,
    point: MazePoint,
}
impl SeekAdjacentPillarOkResult {
    pub(crate) fn new(state: SeekAdjacentPillarOkState, point: MazePoint) -> Self {
        Self { state, point }
    }
    pub fn is_next_pillar(&self) -> bool {
        matches!(self.state, SeekAdjacentPillarOkState::NextPillar)
    }
    pub fn is_outside(&self) -> bool {
        matches!(self.state, SeekAdjacentPillarOkState::Outside)
    }
}

/// 1.&Arc<RwLock<MazePoints>>とMazePointとWallIdentifierを引数として受け取る。
/// 2.MazePointsのall_pillar_seeked_flagがtrueであればエラーを返す。
/// 3.引数として受け取ったMazePointに対応するall_maze_pointsの値がMazePointStatusがWallでWallTypeがPillarで、PillarExtendStatusがInProgressで、WallIdentifierが引数で受け取った値と等しいことを確認する。どれか違っていたらエラーを返す。
/// 4.引数として受け取ったMazePoint(柱)に隣接する柱のMazePointを格納する作業用Vecを生成する。
/// 5.引数として受け取ったMazePointとyが等しく、xが+-2のMazePointを作業用Vecに格納する。
/// 6.引数として受け取ったMazePointとxが等しく、yが+-2のMazePointを作業用Vecに格納する。
/// 7.作業用Vecの長さが4であることを確認する。
/// 8.作業用Vecに格納されたMazePointをランダムで1つ選択する。
/// 9.作業用Vecに格納されたMazePointに対応するall_maze_pointsの値のMazePointStatusがOutsideかWallであるか確認する。Wallの場合はWallTypeがPillarで、PillarExtendStatusがNotCheckedであるか確認する。違う場合はそのMazePointを作業用Vecから削除して8からやり直す。
/// 10.作業用Vecの要素数が0になった場合はエラーを返す。
/// 11.引数として受け取ったMazePointと作業用Vecから選択したMazePointに挟まれたMazePointに対応するall_maze_pointsの値がPathであれば、CandidateWallに変更する。
/// 12.引数として受け取ったMazePointに対応するall_maze_pointsの値のPillarExtendStatusを値をExtendedに変更する。また、作業用Vecから選択したMazePointに対応するall_maze_pointsの値のPillarExtendStatusを値をInProgressに変更する。
/// 13.作業用Vecから選択したMazePointに対応するall_maze_pointsの値のMazePointStatusがOutsideであればエラーを返す。Wallであれば選択したMazePointを返す。
pub fn extend_pillar_to_adjacent_pillar(
    maze_points: &Arc<RwLock<MazePoints>>,
    pillar_point: MazePoint,
    identifier: WallIdentifier,
) -> Result<SeekAdjacentPillarOkResult, Box<dyn std::error::Error + Send + Sync>> {
    let error_msg_common_part = format!(
        " identifier: {} from_pillar {:?}",
        identifier.as_str(),
        pillar_point
    );

    let mut maze_points_guard = maze_points.write().map_err(|_| {
        Box::new(std::io::Error::other(format!(
            "Failed to acquire write lock. {}",
            error_msg_common_part
        ))) as Box<dyn std::error::Error + Send + Sync>
    })?;

    // 2. all_pillar_seeked_flagがtrueであればエラーを返す
    if maze_points_guard.all_pillar_seeked_flag {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "All pillar points have already been sought. {}",
                error_msg_common_part
            ),
        )));
    }

    // 3. 引数として受け取ったMazePointの状態確認
    if let Some(status) = maze_points_guard
        .all_maze_points
        .get(&pillar_point)
        .cloned()
    {
        let status_str = format!("from_pillar_status:{:?}", &status);
        if let MazePointStatus::Wall(WallType::Pillar(extend_status), id) = status {
            if extend_status != PillarExtendStatus::InProgress || id != Some(identifier.clone()) {
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

    // 4-6. 隣接する柱のMazePointを格納する作業用Vecを生成
    let mut adjacent_pillars = Vec::new();

    // x方向の隣接点 (x±2, y)
    if pillar_point.x() >= 2 {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() - 2, pillar_point.y()));
    }
    if pillar_point.x() + 2 < maze_points_guard.x_size {
        adjacent_pillars.push(MazePoint::new(pillar_point.x() + 2, pillar_point.y()));
    }

    // y方向の隣接点 (x, y±2)
    if pillar_point.y() >= 2 {
        adjacent_pillars.push(MazePoint::new(pillar_point.x(), pillar_point.y() - 2));
    }
    if pillar_point.y() + 2 < maze_points_guard.y_size {
        adjacent_pillars.push(MazePoint::new(pillar_point.x(), pillar_point.y() + 2));
    }

    // 7. 作業用Vecの長さが4であることを確認（範囲外の場合は4未満になる）
    // 範囲内であれば4つの隣接点が生成される

    loop {
        // 10. 作業用Vecの要素数が0になった場合はエラーを返す
        if adjacent_pillars.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "No valid adjacent pillar points found. {}",
                    error_msg_common_part
                ),
            )));
        }

        // 8. 作業用Vecに格納されたMazePointをランダムで1つ選択
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..adjacent_pillars.len());
        let selected_point = adjacent_pillars[random_index].clone();
        let selected_point_str = format!("selected_point:{:?}", &selected_point);

        // 9. 選択したMazePointの状態確認
        let is_valid = if let Some(status) = maze_points_guard.all_maze_points.get(&selected_point)
        {
            match status {
                MazePointStatus::Wall(WallType::Outside, _) => true,
                MazePointStatus::Wall(WallType::Pillar(extend_status), _) => {
                    *extend_status == PillarExtendStatus::NotChecked
                }
                _ => false,
            }
        } else {
            false
        };

        if !is_valid {
            // 無効な場合は作業用Vecから削除
            adjacent_pillars.remove(random_index);
            continue;
        }

        // 11. 挟まれたMazePointをCandidateWallに変更
        let middle_point = MazePoint::new(
            (pillar_point.x() + selected_point.x()) / 2,
            (pillar_point.y() + selected_point.y()) / 2,
        );
        let middle_point_str = format!("middle_point:{:?}", &middle_point);

        // MazePointStatusがPathであることを確認
        if let Some(status) = maze_points_guard.all_maze_points.get(&middle_point) {
            let status_str = format!("{} middle_point_status:{:?}", middle_point_str, &status);
            match status {
                MazePointStatus::Path => {
                    // PathであればCandidateWallに変更
                    maze_points_guard.all_maze_points.insert(
                        middle_point,
                        MazePointStatus::Wall(WallType::CandidateWall, Some(identifier.clone())),
                    );
                }
                _ => {
                    // Pathでない場合はエラーを返す
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Middle point is not a Path, {} {}",
                            error_msg_common_part, status_str
                        ),
                    )));
                }
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Middle point not found {}", error_msg_common_part),
            )));
        }

        // 12. PillarExtendStatusを変更
        // 元のピラーをExtendedに変更
        maze_points_guard.all_maze_points.insert(
            pillar_point.clone(),
            MazePointStatus::Wall(
                WallType::Pillar(PillarExtendStatus::Extended),
                Some(identifier.clone()),
            ),
        );

        // 選択したピラーの状態を確認して適切に変更
        if let Some(status) = maze_points_guard
            .all_maze_points
            .get(&selected_point)
            .cloned()
        {
            let status_str = format!("{} selected_point_status:{:?}", selected_point_str, &status);
            match status {
                MazePointStatus::Wall(WallType::Outside, _) => {
                    return Ok(SeekAdjacentPillarOkResult::new(
                        SeekAdjacentPillarOkState::Outside,
                        selected_point,
                    ));
                }
                MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), _) => {
                    // 選択したピラーをInProgressに変更
                    maze_points_guard.all_maze_points.insert(
                        selected_point.clone(),
                        MazePointStatus::Wall(
                            WallType::Pillar(PillarExtendStatus::InProgress),
                            Some(identifier),
                        ),
                    );
                    // 13. Wallであれば選択したMazePointを返す
                    return Ok(SeekAdjacentPillarOkResult::new(
                        SeekAdjacentPillarOkState::NextPillar,
                        selected_point,
                    ));
                }
                _ => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Unexpected pillar state. {} {}",
                            error_msg_common_part, status_str
                        ),
                    )));
                }
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Selected point not found. {}", error_msg_common_part),
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze_point_status::{PillarExtendStatus, WallType};

    #[test]
    fn test_initialize_maze_points_5x5() {
        let result = MazePoints::initialize_maze_points(5, 5);
        assert!(result.is_ok());

        let maze_points_arc = result.unwrap();
        let maze_points = maze_points_arc.read().unwrap();

        // all_maze_points の要素数は25であることを確認
        assert_eq!(maze_points.all_maze_points.len(), 25);

        // 修正後の迷路の状態確認（5x5の場合）:
        // O O O O O   (行0: すべてOutside)
        // O P P P O   (行1: 端はOutside、内部はPath)
        // O P # P O   (行2: 端はOutside、中央(2,2)のみPillar、他はPath)
        // O P P P O   (行3: 端はOutside、内部はPath)
        // O O O O O   (行4: すべてOutside)

        // 外壁はすべてOutside（角も含む）
        let point_0_0 = MazePoint::new(0, 0);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_0_0),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_4_4 = MazePoint::new(4, 4);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_4_4),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_0_4 = MazePoint::new(0, 4);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_0_4),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_4_0 = MazePoint::new(4, 0);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_4_0),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        // 外壁の辺もOutside
        let point_0_1 = MazePoint::new(0, 1);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_0_1),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_1_0 = MazePoint::new(1, 0);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_1_0),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_4_1 = MazePoint::new(4, 1);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_4_1),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        let point_1_4 = MazePoint::new(1, 4);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_1_4),
            Some(MazePointStatus::Wall(WallType::Outside, _))
        ));

        // x=2, y=2 だけが Pillar（内部の偶数座標）
        let point_2_2 = MazePoint::new(2, 2);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_2_2),
            Some(MazePointStatus::Wall(
                WallType::Pillar(PillarExtendStatus::NotChecked),
                None
            ))
        ));

        // 内部の通路部分はPath
        let point_1_1 = MazePoint::new(1, 1);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_1_1),
            Some(MazePointStatus::Path)
        ));

        let point_3_3 = MazePoint::new(3, 3);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_3_3),
            Some(MazePointStatus::Path)
        ));

        let point_1_3 = MazePoint::new(1, 3);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_1_3),
            Some(MazePointStatus::Path)
        ));

        let point_3_1 = MazePoint::new(3, 1);
        assert!(matches!(
            maze_points.all_maze_points.get(&point_3_1),
            Some(MazePointStatus::Path)
        ));

        // pillar_points の要素数は1であることを確認
        assert_eq!(maze_points.pillar_points.len(), 1);

        // pillar_points に x=2, y=2 だけが格納されていることを確認
        assert!(maze_points.pillar_points.contains(&point_2_2));
    }

    #[test]
    fn test_initialize_maze_points_invalid_even_size() {
        // 偶数のサイズでエラーになることを確認
        let result = MazePoints::initialize_maze_points(4, 5);
        assert!(result.is_err());

        let result = MazePoints::initialize_maze_points(5, 4);
        assert!(result.is_err());

        let result = MazePoints::initialize_maze_points(4, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_maze_points_invalid_small_size() {
        // 5未満のサイズでエラーになることを確認
        let result = MazePoints::initialize_maze_points(3, 5);
        assert!(result.is_err());

        let result = MazePoints::initialize_maze_points(5, 3);
        assert!(result.is_err());

        let result = MazePoints::initialize_maze_points(1, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_maze_points_valid_larger_size() {
        // より大きなサイズでも正常に動作することを確認
        let result = MazePoints::initialize_maze_points(7, 7);
        assert!(result.is_ok());

        let maze_points_arc = result.unwrap();
        let maze_points = maze_points_arc.read().unwrap();

        // 要素数は7*7=49
        assert_eq!(maze_points.all_maze_points.len(), 49);

        // 内部の柱の数を確認（(2,2), (2,4), (4,2), (4,4)）
        assert_eq!(maze_points.pillar_points.len(), 4);
    }

    #[test]
    fn test_select_start_pillar_point_success() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();

        let result = select_start_pillar_point(&maze_points, identifier.clone());
        assert!(result.is_ok());

        let point = result.unwrap();

        // 返されたポイントがピラーポイントに含まれていることを確認
        let maze_guard = maze_points.read().unwrap();
        assert!(maze_guard.pillar_points.contains(&point));

        // 状態がInProgressに変更されていることを確認
        if let Some(MazePointStatus::Wall(WallType::Pillar(status), id)) =
            maze_guard.all_maze_points.get(&point)
        {
            assert_eq!(*status, PillarExtendStatus::InProgress);
            assert_eq!(*id, Some(identifier));
        } else {
            panic!("Expected Pillar with InProgress status");
        }
    }

    #[test]
    fn test_select_start_pillar_point_no_available_points() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();

        // 唯一のピラーポイントをInProgressに変更
        {
            let mut maze_guard = maze_points.write().unwrap();
            let point = MazePoint::new(2, 2);
            maze_guard.all_maze_points.insert(
                point,
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );
        }

        // 利用可能なポイントがない場合のエラーを確認
        let result = select_start_pillar_point(&maze_points, identifier);
        assert!(result.is_err());
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_success() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap(); // より大きな迷路に変更
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(4, 4); // 中央の柱を使用

        // まずピラーをInProgressに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_maze_points.insert(
                pillar_point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );
        }

        let result = extend_pillar_to_adjacent_pillar(
            &maze_points,
            pillar_point.clone(),
            identifier.clone(),
        );
        assert!(result.is_ok());

        let seek_result = result.unwrap();

        // 結果に応じたテスト
        if seek_result.is_next_pillar() {
            // 選択されたポイントがピラーポイントに含まれていることを確認
            let maze_guard = maze_points.read().unwrap();
            assert!(maze_guard.pillar_points.contains(&seek_result.point));

            // 元のピラーがExtendedに変更されていることを確認
            if let Some(MazePointStatus::Wall(WallType::Pillar(status), _)) =
                maze_guard.all_maze_points.get(&pillar_point)
            {
                assert_eq!(*status, PillarExtendStatus::Extended);
            } else {
                panic!("Expected original pillar to be Extended");
            }

            // 選択されたピラーがInProgressに変更されていることを確認
            if let Some(MazePointStatus::Wall(WallType::Pillar(status), _)) =
                maze_guard.all_maze_points.get(&seek_result.point)
            {
                assert_eq!(*status, PillarExtendStatus::InProgress);
            } else {
                panic!("Expected selected pillar to be InProgress");
            }

            // 中間点がCandidateWallに変更されていることを確認
            let middle_point = MazePoint::new(
                (pillar_point.x() + seek_result.point.x()) / 2,
                (pillar_point.y() + seek_result.point.y()) / 2,
            );
            if let Some(MazePointStatus::Wall(WallType::CandidateWall, _)) =
                maze_guard.all_maze_points.get(&middle_point)
            {
                // 正常
            } else {
                panic!("Expected middle point to be CandidateWall");
            }
        } else if seek_result.is_outside() {
            // 境界に達した場合も正常
            let maze_guard = maze_points.read().unwrap();

            // 元のピラーがExtendedに変更されていることを確認
            if let Some(MazePointStatus::Wall(WallType::Pillar(status), _)) =
                maze_guard.all_maze_points.get(&pillar_point)
            {
                assert_eq!(*status, PillarExtendStatus::Extended);
            } else {
                panic!("Expected original pillar to be Extended");
            }
        }
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_returns_outside() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(2, 2); // 5x5の唯一のピラー

        // まずピラーをInProgressに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_maze_points.insert(
                pillar_point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );
        }

        let result =
            extend_pillar_to_adjacent_pillar(&maze_points, pillar_point.clone(), identifier);

        // 5x5では隣接ピラーは全て境界なのでOUTSIDEが返される
        assert!(result.is_ok());
        let seek_result = result.unwrap();
        assert!(seek_result.is_outside());
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_all_pillars_sought() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(2, 2);

        // all_pillar_seeked_flagをtrueに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_pillar_seeked_flag = true;
        }

        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("All pillar points have already been sought")
        );
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_invalid_pillar_state() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(2, 2);

        // ピラーがNotCheckedのまま（InProgressではない）
        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not in InProgress state")
        );
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_wrong_identifier() {
        let maze_points = MazePoints::initialize_maze_points(5, 5).unwrap();
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();
        let pillar_point = MazePoint::new(2, 2);

        // identifier1でInProgressに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_maze_points.insert(
                pillar_point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier1),
                ),
            );
        }

        // 異なるidentifier2で拡張を試行
        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("identifier does not match")
        );
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_middle_point_not_path() {
        let maze_points = MazePoints::initialize_maze_points(7, 7).unwrap();
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(2, 2);
        let middle_point = MazePoint::new(3, 2);

        // ピラーをInProgressに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_maze_points.insert(
                pillar_point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );

            // 他の隣接点を無効化（InProgressに設定）
            let other_adjacent = vec![
                MazePoint::new(2, 0), // 上
                MazePoint::new(2, 4), // 下
            ];

            for point in other_adjacent {
                if maze_guard.pillar_points.contains(&point) {
                    maze_guard.all_maze_points.insert(
                        point,
                        MazePointStatus::Wall(
                            WallType::Pillar(PillarExtendStatus::InProgress),
                            Some(identifier.clone()),
                        ),
                    );
                }
            }

            // 中間点を既にWallに設定（Pathではない）
            maze_guard.all_maze_points.insert(
                middle_point,
                MazePointStatus::Wall(WallType::Wall, Some(identifier.clone())),
            );
        }

        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier);

        // 結果をコンソールに表示
        println!("Test result: {:?}", result);

        // 結果は成功(OUTSIDE)またはエラー(Middle point is not a Path)のいずれか
        match result {
            Ok(result) if result.is_outside() => {
                // 境界に達した場合は正常
            }
            Err(e) if e.to_string().contains("Middle point is not a Path") => {
                // 中間点がPathでない場合のエラーも正常
            }
            _ => {
                panic!("Unexpected result: {:?}", result);
            }
        }
    }

    #[test]
    fn test_seek_adjacent_pillar_ok_result_methods() {
        let point = MazePoint::new(2, 2);

        // NEXT_PILLARのテスト
        let next_pillar = SeekAdjacentPillarOkResult::new(
            SeekAdjacentPillarOkState::NextPillar,
            point.clone(),
        );
        assert!(next_pillar.is_next_pillar());
        assert!(!next_pillar.is_outside());

        // OUTSIDEのテスト
        let outside = SeekAdjacentPillarOkResult::new(
            SeekAdjacentPillarOkState::Outside,
            point.clone(),
        );
        assert!(!outside.is_next_pillar());
        assert!(outside.is_outside());
    }

    #[test]
    fn test_extend_pillar_to_adjacent_pillar_get_point() {
        let maze_points = MazePoints::initialize_maze_points(9, 9).unwrap();
        let identifier = WallIdentifier::new();
        let pillar_point = MazePoint::new(4, 4);

        // まずピラーをInProgressに設定
        {
            let mut maze_guard = maze_points.write().unwrap();
            maze_guard.all_maze_points.insert(
                pillar_point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::InProgress),
                    Some(identifier.clone()),
                ),
            );
        }

        let result = extend_pillar_to_adjacent_pillar(&maze_points, pillar_point, identifier);
        assert!(result.is_ok());

        let seek_result = result.unwrap();
        
        // 結果に関係なく、pointフィールドにアクセスできることを確認
        let _returned_point = seek_result.point;
        // pointは結果の状態に応じて適切な値が設定されている
    }
}