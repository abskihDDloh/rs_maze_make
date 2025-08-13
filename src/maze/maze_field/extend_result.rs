use crate::maze::maze_cell::maze_point::point::MazePoint;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewMethodEnforcer {}

impl NewMethodEnforcer {
    /// 新しいエンフォーサーインスタンスを生成します
    ///
    /// # 戻り値
    ///
    /// 新しい `NewMethodEnforcer` インスタンス
    fn new() -> Self {
        NewMethodEnforcer {}
    }
}

/// 隣接する柱への拡張操作の結果状態を表すenum
///
/// この列挙型は柱から隣接する柱への拡張試行の結果を示します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::maze) enum ExtendResult {
    /// 隣接する柱が見つかり、拡張に成功した場合
    NextPillar(MazePoint, NewMethodEnforcer),
    /// 使用済みの柱に到達し、拡張できない場合
    ExtendingPillar(NewMethodEnforcer),
    /// 境界（Outside壁）に到達し、これ以上拡張できない場合
    Outside(NewMethodEnforcer),
}
impl ExtendResult {
    pub fn new_next_pillar(point: MazePoint) -> Self {
        ExtendResult::NextPillar(point, NewMethodEnforcer::new())
    }

    pub fn new_extending_pillar() -> Self {
        ExtendResult::ExtendingPillar(NewMethodEnforcer::new())
    }

    pub fn new_outside() -> Self {
        ExtendResult::Outside(NewMethodEnforcer::new())
    }

    pub fn is_next_pillar(&self) -> bool {
        matches!(self, ExtendResult::NextPillar(_, _))
    }

    /// 隣接する柱が見つかった場合のポイントを取得する
    pub fn point(&self) -> Option<MazePoint> {
        match self {
            ExtendResult::NextPillar(point, _) => Some(*point),
            _ => None,
        }
    }

    /// 拡張状態を取得する
    pub fn is_extending(&self) -> bool {
        matches!(self, ExtendResult::ExtendingPillar(_))
    }

    /// 境界に到達したかどうかを取得する
    pub fn is_outside(&self) -> bool {
        matches!(self, ExtendResult::Outside(_))
    }
}
