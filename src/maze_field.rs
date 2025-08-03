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
    x_size: u32,
    y_size: u32,
    all_maze_points: HashMap<MazePoint, MazePointStatus>,
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
                    if let Some(point_value) = all_maze_points.get_mut(&point) {
                        if point_value == &MazePointStatus::Path {
                            // すでにPathであればPillarに変更
                            *point_value = MazePointStatus::new_notchecked_pillar();
                            if x > 0 && y > 0 && x < x_size - 1 && y < y_size - 1 {
                                wall_start_points.insert(point.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(Arc::new(RwLock::new(MazePoints {
            x_size,
            y_size,
            all_maze_points,
            pillar_points: wall_start_points,
        })))
    }
}

/// 1.&Arc<RwLock<MazePoints>>とWallIdentifierを引数として受け取る。
/// 2.作業用に空のHashSetを作成する。
/// 3.作業用のHashSetの長さがMazePointsのpillar_pointsと同じであるか確認する。同じ場合はエラーを返す。
/// 4.MazePointsのpillar_pointsに格納されている値で、作業用のHashSetに存在しない値から、ランダムで1つMazePointを取得する。
/// 5.作業用のHashSetに取得したMazePointがないことを確認する。ある場合は3からもう1度実施する。
/// 6.取得したMazePointを作業用のHashSetに格納する。
/// 7.取得したMazePointに対応するall_maze_pointsの値がMazePointStatusがWallでWallTypeがPillarで、PillarExtendStatusがNotCheckedであるか確認する。NotCheckedではない場合は3からもう1度実施する。
/// 8.PillarExtendStatusがNotCheckedの場合は、PillarExtendStatusをInProgressに変更する。
/// 9.取得したMazePointを返す。
pub fn get_random_pillar_point(
    maze_points: &Arc<RwLock<MazePoints>>,
    identifier: WallIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error + Send + Sync>> {
    let mut work_set = HashSet::new();

    loop {
        // 書き込みロックを一度だけ取得してアトミックに処理
        let mut maze_points_guard = maze_points.write().map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire write lock",
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        if work_set.len() == maze_points_guard.pillar_points.len() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No more pillar points available",
            )));
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
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No valid pillar points found",
            )));
        }

        let mut rng = rand::rng();
        let random_index = rng.random_range(0..candidates.len());
        let point = candidates[random_index].clone();

        // 再度状態を確認（ダブルチェック）
        if let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), _)) =
            maze_points_guard.all_maze_points.get(&point)
        {
            if *extend_status == PillarExtendStatus::NotChecked {
                // アトミックに状態を変更
                maze_points_guard.all_maze_points.insert(
                    point.clone(),
                    MazePointStatus::Wall(
                        WallType::Pillar(PillarExtendStatus::InProgress),
                        identifier,
                    ),
                );
                return Ok(point);
            }
        }

        // 状態が変わっていた場合は作業セットに追加して再試行
        work_set.insert(point);
        // ロックは自動的に解放されて次のループで再取得
    }
}

/// 1.&Arc<RwLock<MazePoints>>とMazePointとWallIdentifierを引数として受け取る。
/// 2.取得したMazePointがMazePointsのpillar_pointsに含まれているか確認する。含まれていない場合は処理を終了する。
/// 3.取得したMazePointに対応するall_maze_pointsの値がMazePointStatusがWallでWallTypeがPillarで、PillarExtendStatusがInProgressで、WallIdentifierが引数として取得した値と同じであるか確認する。当てはまらない場合は処理を終了する。
/// 4.PillarExtendStatusがInProgressの値をExtendedに変更する。
pub fn change_pillar_status_from_in_progress_to_extended(
    maze_points: &Arc<RwLock<MazePoints>>,
    point: MazePoint,
    identifier: WallIdentifier,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut maze_points_guard = maze_points.write().map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to acquire write lock",
        )) as Box<dyn std::error::Error + Send + Sync>
    })?;

    if !maze_points_guard.pillar_points.contains(&point) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "MazePoint not found in pillar points",
        )));
    }

    // 修正: cloned()を使用
    if let Some(status) = maze_points_guard.all_maze_points.get(&point).cloned() {
        if let MazePointStatus::Wall(WallType::Pillar(extend_status), id) = status {
            if extend_status == PillarExtendStatus::InProgress && id == identifier {
                maze_points_guard.all_maze_points.insert(
                    point,
                    MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::Extended), id),
                );
                Ok(())
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid state or identifier mismatch",
                )))
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MazePoint is not a Pillar",
            )))
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "MazePoint not found",
        )))
    }
}

/// 1.&Arc<RwLock<MazePoints>>とMazePointとWallIdentifierを引数として受け取る。
/// 2.取得したMazePointがMazePointsのpillar_pointsに含まれているか確認する。含まれていない場合は処理を終了する。
/// 3.取得したMazePointに対応するall_maze_pointsの値がMazePointStatusがWallでWallTypeがPillarで、PillarExtendStatusがInProgressで、WallIdentifierが引数として取得した値と同じであるか確認する。当てはまらない場合は処理を終了する。
/// 4.PillarExtendStatusがInProgressの値をNotCheckedに変更する。このとき、WallIdentifier::new("NotChecked".to_string())をセットする。
pub fn change_pillar_status_from_in_progress_to_not_checked(
    maze_points: &Arc<RwLock<MazePoints>>,
    point: MazePoint,
    identifier: WallIdentifier,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut maze_points_guard = maze_points.write().map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to acquire write lock",
        )) as Box<dyn std::error::Error + Send + Sync>
    })?;

    if !maze_points_guard.pillar_points.contains(&point) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "MazePoint not found in pillar points",
        )));
    }

    if let Some(MazePointStatus::Wall(WallType::Pillar(extend_status), id)) =
        maze_points_guard.all_maze_points.get(&point)
    {
        if *extend_status == PillarExtendStatus::InProgress && *id == identifier {
            maze_points_guard.all_maze_points.insert(
                point.clone(),
                MazePointStatus::Wall(
                    WallType::Pillar(PillarExtendStatus::NotChecked),
                    WallIdentifier::new("NotChecked".to_string()),
                ),
            );
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MazePoint is not in InProgress state or identifier does not match",
            )))
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MazePoint is not a Pillar",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze_point_status::WallType;

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
            Some(MazePointStatus::Wall(WallType::Pillar(_), _))
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

        // wall_start_points の要素数は1であることを確認
        assert_eq!(maze_points.pillar_points.len(), 1);

        // wall_start_points に x=2, y=2 だけが格納されていることを確認
        // (2,2)は偶数座標かつ内部の座標なのでwall_start_pointsに含まれる
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
}
