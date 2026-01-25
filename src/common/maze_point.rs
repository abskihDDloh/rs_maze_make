use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
pub struct MazePoint {
    /// X座標（横方向位置）
    x: u64,
    /// Y座標（縦方向位置）
    y: u64,
}

impl MazePoint {
    fn default() -> Self {
        MazePoint { x: 0, y: 0 }
    }

    pub fn new(x: u64, y: u64) -> Self {
        MazePoint { x, y }
    }

    /// X座標を取得します
    ///
    /// この座標点のX座標値（横方向位置）を返します。
    ///
    /// # 戻り値
    ///
    /// X座標値（0以上のu64値）
    pub fn x(&self) -> u64 {
        self.x
    }

    /// Y座標を取得します
    ///
    /// この座標点のY座標値（縦方向位置）を返します。
    ///
    /// # 戻り値
    ///
    /// Y座標値（0以上のu64値）
    pub fn y(&self) -> u64 {
        self.y
    }

    pub fn generate_adjacent_maze_points(&self, distance: u64) -> Vec<MazePoint> {
        // 上下左右に指定された距離だけ離れた位置を生成
        // オーバーフローの場合は最大値、アンダーフローの場合は最小値
        let mut points = HashSet::from([
            MazePoint::new(self.x.saturating_sub(distance), self.y),
            MazePoint::new(self.x.saturating_add(distance), self.y),
            MazePoint::new(self.x, self.y.saturating_sub(distance)),
            MazePoint::new(self.x, self.y.saturating_add(distance)),
        ]);
        // オーバーフロー、アンダーフローの結果として自分の座標が算出された場合は除去する。
        if points.contains(self) {
            points.remove(self);
        }
        points.into_iter().collect()
    }
}

/// from-toを対角線の1つとする長方形に含まれる点をすべて取得する（fromとtoを含む）
pub fn select_between_points(from: &MazePoint, to: &MazePoint) -> Vec<MazePoint> {
    let mut points = Vec::new();

    let (x_start, x_end) = if from.x() < to.x() {
        (from.x(), to.x())
    } else {
        (to.x(), from.x())
    };

    let (y_start, y_end) = if from.y() < to.y() {
        (from.y(), to.y())
    } else {
        (to.y(), from.y())
    };

    for x in x_start..=x_end {
        for y in y_start..=y_end {
            points.push(MazePoint::new(x, y));
        }
    }

    points
}

/// from-toを対角線の1つとする長方形に含まれる点をすべて取得する（fromとtoを除く）
fn select_between_points_exlude_edge(from: &MazePoint, to: &MazePoint) -> Vec<MazePoint> {
    let mut points = select_between_points(from, to);
    points.retain(|p| p != from && p != to);
    points
}

/// from-toを対角線の1つとする長方形に含まれる点のうち、fromに隣接する点のみを取得する。
pub fn select_bitweeb_points_and_from_adjacent(
    from: &MazePoint,
    to: &MazePoint,
    distance: u64,
) -> Vec<MazePoint> {
    let mut points = select_between_points_exlude_edge(from, to);
    let from_adjacent_points = from.generate_adjacent_maze_points(distance);
    points.retain(|p| from_adjacent_points.contains(p));
    points
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn default_is_origin() {
        let point = MazePoint::default();
        assert_eq!(point, MazePoint::new(0, 0));
        assert_eq!(point.x(), 0);
        assert_eq!(point.y(), 0);
    }

    #[test]
    fn generate_adjacent_basic_four_points() {
        let point = MazePoint::new(2, 3);

        let adjacent = point.generate_adjacent_maze_points(1);
        let adjacent_set: HashSet<_> = adjacent.into_iter().collect();

        let expected = HashSet::from([
            MazePoint::new(1, 3),
            MazePoint::new(3, 3),
            MazePoint::new(2, 2),
            MazePoint::new(2, 4),
        ]);

        assert_eq!(adjacent_set, expected);
    }

    //generate_adjacent_maze_points(2)の場合のテスト。
    #[test]
    fn generate_adjacent_distance_two() {
        let point = MazePoint::new(2, 3);

        let adjacent = point.generate_adjacent_maze_points(2);
        let adjacent_set: HashSet<_> = adjacent.into_iter().collect();

        let expected = HashSet::from([
            MazePoint::new(0, 3),
            MazePoint::new(4, 3),
            MazePoint::new(2, 1),
            MazePoint::new(2, 5),
        ]);

        assert_eq!(adjacent_set, expected);
    }
    #[test]
    fn generate_adjacent_saturates_and_excludes_self() {
        let origin = MazePoint::new(0, 0);

        let adjacent = origin.generate_adjacent_maze_points(1);
        let adjacent_set: HashSet<_> = adjacent.into_iter().collect();

        let expected = HashSet::from([MazePoint::new(1, 0), MazePoint::new(0, 1)]);

        assert_eq!(adjacent_set, expected);
        assert!(!adjacent_set.contains(&origin));
    }

    #[test]
    fn generate_adjacent_handles_max_values() {
        let max = u64::MAX;
        let point = MazePoint::new(max, max);

        let adjacent = point.generate_adjacent_maze_points(1);
        let adjacent_set: HashSet<_> = adjacent.into_iter().collect();

        let expected = HashSet::from([MazePoint::new(max - 1, max), MazePoint::new(max, max - 1)]);

        assert_eq!(adjacent_set, expected);
    }

    #[test]
    fn get_between_points_orders_and_includes_bounds() {
        let from = MazePoint::new(1, 1);
        let to = MazePoint::new(2, 2);

        let points = select_between_points(&from, &to);

        let expected = vec![
            MazePoint::new(1, 1),
            MazePoint::new(1, 2),
            MazePoint::new(2, 1),
            MazePoint::new(2, 2),
        ];

        assert_eq!(points, expected);
    }

    #[test]
    fn get_between_points_is_order_independent_for_bounds() {
        let from = MazePoint::new(3, 4);
        let to = MazePoint::new(1, 2);

        let points = select_between_points(&from, &to);
        let expected = vec![
            MazePoint::new(1, 2),
            MazePoint::new(1, 3),
            MazePoint::new(1, 4),
            MazePoint::new(2, 2),
            MazePoint::new(2, 3),
            MazePoint::new(2, 4),
            MazePoint::new(3, 2),
            MazePoint::new(3, 3),
            MazePoint::new(3, 4),
        ];

        assert_eq!(points, expected);
    }

    #[test]
    fn select_between_and_from_adjacent_distance_one() {
        let from = MazePoint::new(2, 2);
        let to = MazePoint::new(4, 4);

        let points = select_bitweeb_points_and_from_adjacent(&from, &to, 1);
        let point_set: HashSet<_> = points.into_iter().collect();

        // distance=1の場合、fromの隣接点は (1,2), (3,2), (2,1), (2,3)
        // from-toの長方形内で端点を除いた点との交差
        let expected = HashSet::from([MazePoint::new(3, 2), MazePoint::new(2, 3)]);

        assert_eq!(point_set, expected);
        assert!(!point_set.contains(&from));
        assert!(!point_set.contains(&to));
    }

    #[test]
    fn select_between_and_from_adjacent_distance_two() {
        let from = MazePoint::new(2, 2);
        let to = MazePoint::new(5, 5);

        let points = select_bitweeb_points_and_from_adjacent(&from, &to, 2);
        let point_set: HashSet<_> = points.into_iter().collect();

        // distance=2の場合、fromの隣接点は (0,2), (4,2), (2,0), (2,4)
        // from-toの長方形内で端点を除いた点との交差
        let expected = HashSet::from([MazePoint::new(4, 2), MazePoint::new(2, 4)]);

        assert_eq!(point_set, expected);
    }

    #[test]
    fn select_between_and_from_adjacent_no_intersection() {
        let from = MazePoint::new(1, 1);
        let to = MazePoint::new(2, 2);

        let points = select_bitweeb_points_and_from_adjacent(&from, &to, 1);
        let point_set: HashSet<_> = points.into_iter().collect();

        // 小さい長方形で、fromの隣接点がfrom-toの矩形内に存在しない場合
        let expected = HashSet::from([MazePoint::new(2, 1), MazePoint::new(1, 2)]);
        assert_eq!(point_set, expected);
    }
}
