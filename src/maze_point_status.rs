use std::{
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

/// 内部用の識別子構造体（非公開）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WallIdentifier {
    tid_id: thread::ThreadId,
    unix_time_nanos: i64,
}

impl WallIdentifier {
    /// 新しい識別子を生成する。
    /// 生成された識別子は、スレッドIDと現在のUNIX時間(ナノ秒)を組み合わせた形式になります。
    /// 例えば、"ThreadId(0x7f8b3c0c4c40)_1633036800"
    pub(crate) fn new() -> Self {
        // 10nsのスリープを入れて、UNIX時間の重複を回避する。
        use std::time::Duration;
        thread::sleep(Duration::new(0, 10));
        let tid_id: thread::ThreadId = thread::current().id();
        // SystemTimeを使用してナノ秒精度で取得
        let unix_time_nanos: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as i64;
        WallIdentifier {
            tid_id,
            unix_time_nanos,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{:?}_{}", self.tid_id, self.unix_time_nanos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PillarExtendStatus {
    /// 拡張処理未済
    NotChecked,
    /// 拡張処理中
    InProgress,
    /// 拡張された柱
    Extended,
}
impl PillarExtendStatus {
    pub fn is_not_checked(&self) -> bool {
        matches!(self, PillarExtendStatus::NotChecked)
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self, PillarExtendStatus::InProgress)
    }

    pub fn is_extended(&self) -> bool {
        matches!(self, PillarExtendStatus::Extended)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WallType {
    ///外壁
    Outside,
    ///柱
    Pillar(PillarExtendStatus),
    ///生成された壁
    Wall,
    ///新しい候補壁
    CandidateWall,
}
impl WallType {
    pub fn is_pillar(&self) -> bool {
        matches!(self, WallType::Pillar(_))
    }

    pub fn is_outside(&self) -> bool {
        matches!(self, WallType::Outside)
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, WallType::Wall)
    }

    pub fn is_candidate_wall(&self) -> bool {
        matches!(self, WallType::CandidateWall)
    }

    pub fn get_wall_extens_status(&self) -> Option<&PillarExtendStatus> {
        match self {
            WallType::Pillar(status) => Some(status),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MazePointStatus {
    ///通路
    Path,
    ///壁
    Wall(WallType, Option<WallIdentifier>),
}

impl MazePointStatus {
    /// new_wall()に設定する識別子を生成する。
    pub fn new_identifier() -> WallIdentifier {
        WallIdentifier::new()
    }

    pub fn new_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(WallType::Outside, Some(identifier))
    }

    pub fn new_notchecked_pillar() -> Self {
        MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None)
    }

    pub fn is_path(&self) -> bool {
        matches!(self, MazePointStatus::Path)
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, MazePointStatus::Wall(_, _))
    }

    pub fn get_wall_type(&self) -> Option<&WallType> {
        match self {
            MazePointStatus::Path => None,
            MazePointStatus::Wall(wall_type, _) => Some(wall_type),
        }
    }

    pub fn get_wall_identifier(&self) -> Option<WallIdentifier> {
        match self {
            MazePointStatus::Path => None,
            MazePointStatus::Wall(_wall_type, identifier) => identifier.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_point_status_path() {
        let status = MazePointStatus::Path;
        assert!(status.is_path());
        assert!(!status.is_wall());
    }

    #[test]
    fn test_wall_identifier_new_and_as_str() {
        let identifier1 = WallIdentifier::new();
        let identifier2 = WallIdentifier::new();

        // 異なる識別子が生成されることを確認
        assert_ne!(identifier1.as_str(), identifier2.as_str());

        // 識別子が期待する形式であることを確認
        let id_str = identifier1.as_str();
        assert!(id_str.contains("ThreadId"));
        assert!(id_str.contains("_"));
    }

    #[test]
    fn test_maze_point_status_new_outside_wall() {
        let identifier = WallIdentifier::new();
        let status = MazePointStatus::new_outside_wall(identifier.clone());

        assert!(status.is_wall());
        assert!(!status.is_path());

        match status {
            MazePointStatus::Wall(WallType::Outside, Some(id)) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Wall with Outside type"),
        }
    }

    #[test]
    fn test_maze_point_status_new_notchecked_pillar() {
        let status = MazePointStatus::new_notchecked_pillar();

        match status {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None) => {
                // 正常: NotCheckedピラーは識別子がNone
            }
            _ => panic!("Expected Wall with NotChecked Pillar and None identifier"),
        }
    }

    #[test]
    fn test_pillar_extend_status_checks() {
        let not_checked = PillarExtendStatus::NotChecked;
        let in_progress = PillarExtendStatus::InProgress;
        let extended = PillarExtendStatus::Extended;

        assert!(not_checked.is_not_checked());
        assert!(!not_checked.is_in_progress());
        assert!(!not_checked.is_extended());

        assert!(!in_progress.is_not_checked());
        assert!(in_progress.is_in_progress());
        assert!(!in_progress.is_extended());

        assert!(!extended.is_not_checked());
        assert!(!extended.is_in_progress());
        assert!(extended.is_extended());
    }

    #[test]
    fn test_wall_type_methods() {
        let outside = WallType::Outside;
        let pillar = WallType::Pillar(PillarExtendStatus::NotChecked);
        let wall = WallType::Wall;
        let candidate = WallType::CandidateWall;

        assert!(outside.is_outside());
        assert!(!outside.is_pillar());
        assert!(!outside.is_wall());
        assert!(!outside.is_candidate_wall());

        assert!(pillar.is_pillar());
        assert!(!pillar.is_outside());
        assert!(!pillar.is_wall());
        assert!(!pillar.is_candidate_wall());
        assert!(pillar.get_wall_extens_status().is_some());
        assert_eq!(
            *pillar.get_wall_extens_status().unwrap(),
            PillarExtendStatus::NotChecked
        );

        assert!(wall.is_wall());
        assert!(!wall.is_pillar());
        assert!(!wall.is_outside());
        assert!(!wall.is_candidate_wall());

        assert!(candidate.is_candidate_wall());
        assert!(!candidate.is_pillar());
        assert!(!candidate.is_outside());
        assert!(!candidate.is_wall());
    }

    #[test]
    fn test_maze_point_status_get_methods() {
        let path = MazePointStatus::Path;
        assert!(path.get_wall_type().is_none());
        assert!(path.get_wall_identifier().is_none());

        let identifier = WallIdentifier::new();
        let wall_status = MazePointStatus::Wall(WallType::Outside, Some(identifier.clone()));

        assert!(wall_status.get_wall_type().is_some());
        assert_eq!(*wall_status.get_wall_type().unwrap(), WallType::Outside);

        assert!(wall_status.get_wall_identifier().is_some());
        assert_eq!(
            wall_status.get_wall_identifier().unwrap().as_str(),
            identifier.as_str()
        );
    }

    #[test]
    fn test_wall_identifier_uniqueness() {
        let mut identifiers = Vec::new();

        // 複数の識別子を生成
        for _ in 0..10 {
            identifiers.push(WallIdentifier::new());
        }

        // 全て異なることを確認
        for i in 0..identifiers.len() {
            for j in i + 1..identifiers.len() {
                assert_ne!(identifiers[i].as_str(), identifiers[j].as_str());
            }
        }
    }

    #[test]
    fn test_new_identifier_method() {
        let identifier1 = MazePointStatus::new_identifier();
        let identifier2 = MazePointStatus::new_identifier();

        // 異なる識別子が生成されることを確認
        assert_ne!(identifier1.as_str(), identifier2.as_str());
    }
}
