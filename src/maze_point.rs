use std::{collections::HashSet, hash::Hash};

///迷路内のXY座標を表す構造体。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct MazePoint {
    x: u32,
    y: u32,
}

impl MazePoint {
    pub fn new(x: u32, y: u32) -> Self {
        MazePoint { x, y }
    }
}

impl MazePoint {
    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn generate_adjacent_maze_points(&self, distance: u32) -> HashSet<MazePoint> {
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
        points
    }
}

/// 2つのMazePointが与えられたとき、その間にあるMazePointの一覧を取得する。
pub fn get_between_points(from: &MazePoint, to: &MazePoint) -> Vec<MazePoint> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_between_points_horizontal_line() {
        // 水平線のテスト
        let from = MazePoint::new(1, 3);
        let to = MazePoint::new(5, 3);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 5); // 1, 2, 3, 4, 5

        let expected = vec![
            MazePoint::new(1, 3),
            MazePoint::new(2, 3),
            MazePoint::new(3, 3),
            MazePoint::new(4, 3),
            MazePoint::new(5, 3),
        ];

        for expected_point in &expected {
            assert!(
                points.contains(expected_point),
                "Expected point {:?} not found in {:?}",
                expected_point,
                points
            );
        }
    }

    #[test]
    fn test_get_between_points_vertical_line() {
        // 垂直線のテスト
        let from = MazePoint::new(3, 1);
        let to = MazePoint::new(3, 5);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 5); // 1, 2, 3, 4, 5

        let expected = vec![
            MazePoint::new(3, 1),
            MazePoint::new(3, 2),
            MazePoint::new(3, 3),
            MazePoint::new(3, 4),
            MazePoint::new(3, 5),
        ];

        for expected_point in &expected {
            assert!(
                points.contains(expected_point),
                "Expected point {:?} not found in {:?}",
                expected_point,
                points
            );
        }
    }

    #[test]
    fn test_get_between_points_rectangle() {
        // 矩形領域のテスト
        let from = MazePoint::new(1, 1);
        let to = MazePoint::new(3, 2);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 6); // 3x2の矩形

        let expected = vec![
            MazePoint::new(1, 1),
            MazePoint::new(1, 2),
            MazePoint::new(2, 1),
            MazePoint::new(2, 2),
            MazePoint::new(3, 1),
            MazePoint::new(3, 2),
        ];

        for expected_point in &expected {
            assert!(
                points.contains(expected_point),
                "Expected point {:?} not found in {:?}",
                expected_point,
                points
            );
        }
    }

    #[test]
    fn test_get_between_points_same_point() {
        // 同じ点のテスト
        let point = MazePoint::new(5, 5);
        let points = get_between_points(&point, &point);

        assert_eq!(points.len(), 1);
        assert_eq!(points[0], point);
    }

    #[test]
    fn test_get_between_points_reversed_coordinates() {
        // 座標順序を逆にしたテスト
        let from = MazePoint::new(5, 7);
        let to = MazePoint::new(2, 3);
        let points = get_between_points(&from, &to);

        // 逆順でも同じ結果になることを確認
        let points_reversed = get_between_points(&to, &from);

        assert_eq!(points.len(), points_reversed.len());
        assert_eq!(points.len(), 20); // 4x5の矩形

        // 両方向で同じ点が含まれることを確認
        for point in &points {
            assert!(
                points_reversed.contains(point),
                "Point {:?} should be in both results",
                point
            );
        }
    }

    #[test]
    fn test_get_between_points_adjacent_points() {
        // 隣接する点のテスト（距離1）
        let from = MazePoint::new(3, 3);
        let to = MazePoint::new(4, 3);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 2);
        assert!(points.contains(&from));
        assert!(points.contains(&to));
    }

    #[test]
    fn test_get_between_points_pillar_distance() {
        // 迷路の柱間距離（距離2）のテスト
        let from = MazePoint::new(2, 2);
        let to = MazePoint::new(2, 4);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 3);

        let expected = vec![
            MazePoint::new(2, 2), // 開始柱
            MazePoint::new(2, 3), // 中間点
            MazePoint::new(2, 4), // 終了柱
        ];

        for expected_point in &expected {
            assert!(
                points.contains(expected_point),
                "Expected point {:?} not found in {:?}",
                expected_point,
                points
            );
        }

        // 順序の確認（座標順になっているはず）
        assert_eq!(points, expected);
    }

    #[test]
    fn test_get_between_points_diagonal_rectangle() {
        // 対角線方向の矩形テスト
        let from = MazePoint::new(0, 0);
        let to = MazePoint::new(2, 2);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 9); // 3x3の矩形

        // 各行と列の点が含まれることを確認
        for x in 0..=2 {
            for y in 0..=2 {
                let expected_point = MazePoint::new(x, y);
                assert!(
                    points.contains(&expected_point),
                    "Expected point {:?} not found in {:?}",
                    expected_point,
                    points
                );
            }
        }
    }

    #[test]
    fn test_get_between_points_large_coordinates() {
        // 大きな座標でのテスト
        let from = MazePoint::new(100, 200);
        let to = MazePoint::new(102, 201);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 6); // 3x2の矩形

        // 境界点の確認
        assert!(points.contains(&from));
        assert!(points.contains(&to));
        assert!(points.contains(&MazePoint::new(101, 200)));
        assert!(points.contains(&MazePoint::new(100, 201)));
    }

    #[test]
    fn test_get_between_points_zero_coordinates() {
        // 座標0を含むテスト
        let from = MazePoint::new(0, 0);
        let to = MazePoint::new(1, 1);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 4); // 2x2の矩形

        let expected = vec![
            MazePoint::new(0, 0),
            MazePoint::new(0, 1),
            MazePoint::new(1, 0),
            MazePoint::new(1, 1),
        ];

        for expected_point in &expected {
            assert!(
                points.contains(expected_point),
                "Expected point {:?} not found in {:?}",
                expected_point,
                points
            );
        }
    }

    #[test]
    fn test_get_between_points_single_coordinate_change() {
        // 一つの座標のみが変化するテスト

        // X座標のみ変化
        let from = MazePoint::new(1, 5);
        let to = MazePoint::new(4, 5);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 4);
        for point in &points {
            assert_eq!(point.y(), 5, "Y coordinate should remain constant");
        }

        // Y座標のみ変化
        let from = MazePoint::new(5, 1);
        let to = MazePoint::new(5, 4);
        let points = get_between_points(&from, &to);

        assert_eq!(points.len(), 4);
        for point in &points {
            assert_eq!(point.x(), 5, "X coordinate should remain constant");
        }
    }

    #[test]
    fn test_get_between_points_order_independence() {
        // 引数の順序に依存しないことを確認
        let point1 = MazePoint::new(3, 7);
        let point2 = MazePoint::new(8, 2);

        let points1 = get_between_points(&point1, &point2);
        let points2 = get_between_points(&point2, &point1);

        assert_eq!(points1.len(), points2.len());

        // 同じ点が含まれることを確認（順序は異なる可能性がある）
        for point in &points1 {
            assert!(
                points2.contains(point),
                "Point {:?} should be in both results",
                point
            );
        }

        for point in &points2 {
            assert!(
                points1.contains(point),
                "Point {:?} should be in both results",
                point
            );
        }
    }

    #[test]
    fn test_get_between_points_maze_generation_scenario() {
        // 迷路生成の実際のシナリオをテスト

        // 偶数座標の柱（迷路生成でよく使われる）
        let pillar1 = MazePoint::new(2, 2);
        let pillar2 = MazePoint::new(6, 2);
        let points = get_between_points(&pillar1, &pillar2);

        assert_eq!(points.len(), 5); // 2, 3, 4, 5, 6

        // 中間点の確認
        assert!(points.contains(&MazePoint::new(3, 2)));
        assert!(points.contains(&MazePoint::new(4, 2)));
        assert!(points.contains(&MazePoint::new(5, 2)));

        // 開始点と終了点の確認
        assert!(points.contains(&pillar1));
        assert!(points.contains(&pillar2));
    }

    #[test]
    fn test_get_between_points_boundary_values() {
        // 境界値のテスト

        // 最小値
        let from = MazePoint::new(0, 0);
        let to = MazePoint::new(0, 0);
        let points = get_between_points(&from, &to);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0], from);

        // 大きな値での境界テスト
        let from = MazePoint::new(u32::MAX - 2, u32::MAX - 2);
        let to = MazePoint::new(u32::MAX, u32::MAX);
        let points = get_between_points(&from, &to);
        assert_eq!(points.len(), 9); // 3x3の矩形

        // 境界点が含まれることを確認
        assert!(points.contains(&from));
        assert!(points.contains(&to));
    }
}
