/// 外壁の具体的な種類を表す列挙型
///
/// 外壁にはスタートポイントとして機能するものと、単純な境界壁があります。
/// この分類により、迷路生成の開始点を適切に管理できます。
///
/// # 外壁の種類
///
/// - [`StartPoint`](OutsideWallType::StartPoint): 迷路生成の開始点として使用される外壁
/// - [`JustWall`](OutsideWallType::JustWall): 通常の境界壁（開始点ではない）
///
/// # 使用例
///
/// ```rust
/// # use crate::maze::maze_cell::wall::wall_type::{OutsideWallType, WallType, ExtendStatus};
/// // スタートポイント外壁
/// let start_point = OutsideWallType::StartPoint;
/// let start_wall = WallType::Outside(start_point, ExtendStatus::NotChecked);
///
/// // 通常の外壁
/// let just_wall = OutsideWallType::JustWall;
/// let boundary_wall = WallType::Outside(just_wall, ExtendStatus::NotChecked);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutsideWallType {
    /// 迷路生成の開始点
    ///
    /// この外壁は迷路生成アルゴリズムの開始点として使用されます。
    ///
    /// # 特徴
    ///
    /// - 迷路の境界上の偶数座標（0以外）に配置される
    /// - 壁の拡張処理の起点として機能する
    /// - `NotChecked` から `Extending` 状態に変換可能
    /// - 内部の柱や他の外壁への接続を試行する
    StartPoint,

    /// 通常の境界壁
    ///
    /// 迷路の境界を形成する一般的な外壁です。
    ///
    /// # 特徴
    ///
    /// - 迷路の境界を形成する固定的な壁
    /// - 開始点としては使用されない
    /// - 常に壁として機能し、通路になることはない
    /// - 拡張処理の対象外
    JustWall,
}
