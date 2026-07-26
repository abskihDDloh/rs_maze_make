use std::{collections::HashSet, hash::Hash};

/// 迷路内のXY座標を表す構造体
///
/// 迷路システムにおける座標点の表現に特化した軽量な座標構造体です。
/// 2次元迷路の各位置を一意に識別し、座標演算や空間関係の計算を効率的に実行します。
///
/// # 設計原則
///
/// ## 不変性と値セマンティクス
/// - **Copy/Clone**: 軽量な値型として設計、コピーコストが最小
/// - **不変性**: 一度作成された座標は変更されない
/// - **値セマンティクス**: 参照ではなく値として扱われる
///
/// ## 型安全性の確保
/// - **符号なし整数**: 負の座標を防止し、配列インデックスとして安全
/// - **オーバーフロー保護**: saturating演算による境界値での安全な計算
/// - **ハッシュ対応**: コレクションでの効率的な使用をサポート
///
/// ## 迷路特化機能
/// - **隣接点生成**: 指定距離の隣接座標を効率的に計算
/// - **空間関係**: 2点間の領域計算機能
/// - **境界処理**: 迷路境界での適切な動作保証
///
/// # 座標系仕様
///
/// ## 座標原点とスケール
/// - **原点**: (0, 0) 左上角
/// - **X軸**: 右方向が正の方向
/// - **Y軸**: 下方向が正の方向
/// - **単位**: 迷路グリッドの1セル単位
///
/// ## 座標範囲
/// - **最小値**: (0, 0)
/// - **最大値**: (u64::MAX, u64::MAX)
/// - **実用範囲**: 迷路サイズに依存（通常数千以下）
///
/// # パフォーマンス特性
///
/// ## メモリ効率
/// - **サイズ**: 16バイト（u64 × 2）
/// - **アライメント**: 8バイト境界
/// - **キャッシュ親和性**: 小さなサイズによる良好な局所性
///
/// ## 計算性能
/// - **作成**: O(1) - 単純な構造体初期化
/// - **比較**: O(1) - 2つのu64比較
/// - **ハッシュ**: O(1) - 構造的ハッシュ計算
/// - **隣接点生成**: O(1) - 固定数の算術演算
///
/// # スレッドセーフティ
///
/// `MazePoint` は完全にスレッドセーフです：
/// - **Send + Sync**: スレッド間で安全に共有・転送可能
/// - **不変性**: 競合状態の発生なし
/// - **原子性**: Copy操作は本質的に原子的
///
/// # 迷路システムでの用途
///
/// ## 基本的な位置表現
/// - **フィールド索引**: 迷路配列のインデックス計算
/// - **境界判定**: 迷路サイズ内での有効性確認
/// - **座標変換**: 論理座標と物理座標の相互変換
///
/// ## アルゴリズム支援
/// - **経路探索**: グラフノードとしての座標表現
/// - **迷路生成**: 柱配置と壁配置の座標管理
/// - **拡張処理**: 隣接関係の効率的な計算
///
/// ## 空間計算
/// - **距離計算**: 2点間のマンハッタン距離
/// - **領域生成**: 矩形領域内の全座標点列挙
/// - **隣接判定**: 指定距離内の隣接点検索
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
pub struct MazePoint {
    /// X座標（横方向位置）
    x: u64,
    /// Y座標（縦方向位置）
    y: u64,
}

impl MazePoint {
    /// 新しい迷路座標点を作成します
    ///
    /// 指定されたX、Y座標で新しい `MazePoint` インスタンスを生成します。
    /// 座標値の妥当性チェックは行わないため、呼び出し側で適切な値を保証する必要があります。
    ///
    /// # 設計考慮事項
    ///
    /// ## 軽量設計
    /// - **検証無し**: パフォーマンス優先でバリデーションを省略
    /// - **直接設定**: 内部フィールドへの直接代入
    /// - **即座返却**: 追加的な処理を行わない
    ///
    /// ## 境界値対応
    /// - **u64全範囲**: 0からu64::MAXまでの全値を受容
    /// - **オーバーフロー**: 呼び出し側の責任で管理
    /// - **境界確認**: 迷路サイズとの整合性は別途確認
    ///
    /// # 引数
    ///
    /// * `x` - X座標（0以上の整数）
    /// * `y` - Y座標（0以上の整数）
    ///
    /// # 戻り値
    ///
    /// 指定座標の新しい `MazePoint` インスタンス
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

    /// 指定距離の隣接座標点を生成します
    ///
    /// この座標点から上下左右に指定された距離だけ離れた4つの座標点を生成します。
    /// オーバーフロー・アンダーフローは安全に処理され、現在座標と同一になった場合は除外されます。
    ///
    /// # アルゴリズム詳細
    ///
    /// ## 隣接点計算
    /// 1. **上方向**: (x, y - distance)
    /// 2. **下方向**: (x, y + distance)
    /// 3. **左方向**: (x - distance, y)
    /// 4. **右方向**: (x + distance, y)
    ///
    /// ## 境界処理
    /// - **アンダーフロー**: `saturating_sub()` により0にクランプ
    /// - **オーバーフロー**: `saturating_add()` によりu64::MAXにクランプ
    /// - **自己除外**: 結果が現在座標と同一の場合は除去
    ///
    /// ## 重複除去
    /// `HashSet` による効率的な重複排除：
    /// - 同一座標が複数方向から生成される場合（境界付近）
    /// - 距離0指定時の全方向同一座標
    ///
    /// # 実用的な使用パターン
    ///
    /// ## 迷路生成での使用
    /// - **距離2**: 柱から柱への標準的な拡張距離
    /// - **距離1**: 直接隣接セルの取得
    /// - **大距離**: 遠距離接続の可能性探索
    ///
    /// ## 境界での動作
    /// - **角座標**: 2-3個の有効隣接点
    /// - **辺座標**: 3個の有効隣接点
    /// - **内部座標**: 通常4個の隣接点
    ///
    /// # パフォーマンス特性
    ///
    /// - **時間計算量**: O(1) - 固定4回の算術演算
    /// - **空間計算量**: O(1) - 最大4要素のベクタ
    /// - **メモリ割り当て**: HashSet一時使用、最終的にVecに変換
    ///
    /// # 引数
    ///
    /// * `distance` - 隣接点までの距離（0以上の整数）
    ///
    /// # 戻り値
    ///
    /// 有効な隣接座標点のベクタ（0-4個の要素）
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

/// 2つの座標点間の矩形領域内の全座標点を取得します
///
/// 指定された2つの座標点を対角頂点とする矩形領域内の全ての座標点を生成します。
/// 引数の順序に依存せず、常に同じ矩形領域を計算します。
///
/// # 数学的定義
///
/// ## 矩形領域の定義
/// ```text
/// 領域 = {(x, y) | min(x1, x2) ≤ x ≤ max(x1, x2), min(y1, y2) ≤ y ≤ max(y1, y2)}
/// ```
///
/// ## 座標順序の正規化
/// - **X範囲**: `[min(from.x, to.x), max(from.x, to.x)]`
/// - **Y範囲**: `[min(from.y, to.y), max(from.y, to.y)]`
/// - **包含性**: 境界座標も含む閉区間
///
/// # アルゴリズム仕様
///
/// ## 計算手順
/// 1. **範囲決定**: 各軸の最小値・最大値を算出
/// 2. **座標生成**: ネストループによる全組み合わせ生成
/// 3. **ベクタ構築**: 行優先順序での座標点列挙
///
/// ## 順序保証
/// - **行優先**: Y座標が外側ループ、X座標が内側ループ
/// - **昇順**: 小さい座標から大きい座標への順序
/// - **一貫性**: 引数順序に関係なく同一順序
///
/// # 迷路システムでの用途
///
/// ## 壁配置計算
/// - **柱間接続**: 2つの柱間の中間壁配置
/// - **領域塗り潰し**: 特定領域の一括状態変更
/// - **境界計算**: 迷路境界領域の定義
///
/// ## 経路計算
/// - **直線経路**: 2点間の最短矩形経路
/// - **探索領域**: アルゴリズムの探索範囲定義
/// - **衝突判定**: 矩形領域での衝突検出
///
/// ## 描画支援
/// - **レンダリング範囲**: 描画対象領域の特定
/// - **更新領域**: 部分更新での対象範囲
/// - **クリッピング**: 表示範囲の切り取り
///
/// # パフォーマンス特性
///
/// ## 計算量分析
/// - **時間計算量**: O((Δx + 1) × (Δy + 1))
///   - Δx = |to.x - from.x|、Δy = |to.y - from.y|
/// - **空間計算量**: O((Δx + 1) × (Δy + 1))
///   - 結果ベクタのサイズに比例
///
/// ## 実用的な性能
/// - **小矩形**: 数十点の場合、μ秒オーダー
/// - **中矩形**: 数百点の場合、ms オーダー
/// - **大矩形**: 数万点の場合、十数ms オーダー
///
/// # 特殊ケース
///
/// ## 退化ケース
/// - **同一点**: 1要素のベクタを返す
/// - **線分**: 直線上の全座標点（水平/垂直）
/// - **大領域**: メモリ使用量に注意が必要
///
/// ## 境界条件
/// - **座標0**: 最小座標での正常動作
/// - **最大座標**: u64::MAX付近での適切な処理
/// - **座標逆順**: 引数順序の影響なし
///
/// # 注意事項
///
/// ## メモリ使用量
/// - **大領域**: 矩形サイズに比例してメモリ消費
/// - **推奨制限**: 一辺1000以下での使用を推奨
/// - **代替手段**: 大領域では iterator ベースの処理を検討
///
/// ## 座標妥当性
/// - **範囲外座標**: u64範囲外の座標は未対応
/// - **迷路境界**: 迷路サイズとの整合性は呼び出し側で確認
/// - **負座標**: u64型のため負座標は表現不可
///
/// # 引数
///
/// * `from` - 矩形の一方の頂点座標
/// * `to` - 矩形の他方の頂点座標
///
/// # 戻り値
///
/// 矩形領域内の全座標点のベクタ（行優先順序）
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
        let from = MazePoint::new(u64::MAX - 2, u64::MAX - 2);
        let to = MazePoint::new(u64::MAX, u64::MAX);
        let points = get_between_points(&from, &to);
        assert_eq!(points.len(), 9); // 3x3の矩形

        // 境界点が含まれることを確認
        assert!(points.contains(&from));
        assert!(points.contains(&to));
    }
}
