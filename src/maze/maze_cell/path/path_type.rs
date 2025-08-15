/// 通路の種類を表す列挙型
///
/// - `RESOLVED_PATH`: 経路探索などで「解決済み」となった通路
/// - `NOT_RESOLVED_PATH`: まだ経路探索されていない通常の通路
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathType {
    /// 経路探索の始点もしくは終点。
    StartOrEnd,
    /// 経路探索で到達済みの通路
    ResolvedPath,
    /// 未到達・未探索の通路
    NotResolvedPath,
}
