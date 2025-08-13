
/// 柱の拡張状態を表す列挙型
///
/// 迷路生成アルゴリズムにおいて、各柱の処理状態を管理するために使用されます。
/// この状態遷移により、迷路生成の進行状況を追跡し、マルチスレッド環境での
/// 安全な処理を実現します。
///
/// # 状態遷移図
///
/// ```text
/// [NotChecked] ---> [Extending]
///      ↑               ↓
///   初期状態      拡張処理中
/// ```
///
/// # 各状態の説明
///
/// - [`NotChecked`](ExtendStatus::NotChecked): 初期状態。まだ処理されていない柱
/// - [`Extending`](ExtendStatus::Extending): 拡張処理中。他の柱への接続を試行している状態
///
/// # 識別子との関係
///
/// - `NotChecked` 状態の柱は識別子を持ちません（`None`）
/// - `Extending` 状態の柱は処理を開始したスレッドの識別子を持ちます（`Some(WallIdentifier)`）
///
/// # 使用例
///
/// ```rust
/// # use crate::maze_point_status::ExtendStatus;
/// // 初期状態の柱
/// let initial_status = ExtendStatus::NotChecked;
/// assert!(matches!(initial_status, ExtendStatus::NotChecked));
///
/// // 拡張処理開始
/// let extending_status = ExtendStatus::Extending;
/// assert!(matches!(extending_status, ExtendStatus::Extending));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtendStatus {
    /**
     * 拡張処理未開始
     *
     * この状態の柱は：
     * - まだどのスレッドからも処理されていない
     * - 迷路生成の開始点候補として選択可能
     * - 他のスレッドが処理を開始する可能性がある
     */
    NotChecked,

    /**
     * 拡張処理中
     *
     * この状態の柱は：
     * - 特定のスレッドが処理を開始している
     * - 他の隣接柱への拡張を試行中(あるいは試行済み)
     * - 他のスレッドからは処理対象外となる
     */
    Extending,
}
