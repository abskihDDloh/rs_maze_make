use crate::maze::maze_cell::maze_point::point::MazePoint;

/// 隣接する柱への拡張操作の結果状態を表すenum
///
/// この列挙型は柱から隣接する柱への拡張試行の結果を示します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ExtendResult {
    /// 隣接する柱が見つかり、拡張に成功した場合
    NextPillar(MazePoint),
    /// 使用済みの柱に到達し、拡張できない場合
    ExtendingPillar,
    /// 境界（Outside壁）に到達し、これ以上拡張できない場合
    Outside,
}
