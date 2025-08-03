use std::thread;

/// 内部用の識別子構造体（非公開）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WallIdentifier(String);

impl WallIdentifier {
    pub(crate) fn new(identifier: String) -> Self {
        WallIdentifier(identifier)
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
    Wall(WallType, WallIdentifier),
}

impl MazePointStatus {
    /// new_wall()に設定する識別子を生成する。
    /// この関数は、スレッドIDとUNIX時間を組み合わせた文字列を生成します。
    /// 生成された識別子は、スレッドIDと現在のUNIX時間を組み合わせた形式になります。
    /// 例えば、"ThreadId(0x7f8b3c0c4c40)_1633036800"
    pub fn new_identifier() -> WallIdentifier {
        let tid_id = thread::current().id();
        let unix_time = chrono::Utc::now().timestamp();
        let identifier = format!("{:?}_{}", tid_id, unix_time);
        WallIdentifier::new(identifier)
    }

    pub fn new_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(WallType::Outside, identifier)
    }

    pub fn new_notchecked_pillar() -> Self {
        MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::NotChecked),
            WallIdentifier::new("NotChecked".to_string()),
        )
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
            MazePointStatus::Wall(_wall_type, identifier) => Some(identifier.clone()),
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
        let identifier = WallIdentifier::new("test_id".to_string());
        assert_eq!(identifier.as_str(), "test_id");
    }

    #[test]
    fn test_maze_point_status_new_outside_wall() {
        let identifier = WallIdentifier::new("unique_test_id".to_string());
        let status = MazePointStatus::new_outside_wall(identifier.clone());

        assert!(status.is_wall());
        assert!(!status.is_path());

        match status {
            MazePointStatus::Wall(WallType::Outside, id) => {
                assert_eq!(id.as_str(), identifier.as_str());
            }
            _ => panic!("Expected Wall with Outside type"),
        }
    }

    #[test]
    fn test_maze_point_status_new_notchecked_pillar() {
        let status = MazePointStatus::new_notchecked_pillar();

        match status {
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), id) => {
                assert_eq!(id.as_str(), "NotChecked");
            }
            _ => panic!("Expected Wall with NotChecked Pillar"),
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

        assert!(pillar.is_pillar());
        assert!(!pillar.is_outside());
        assert!(pillar.get_wall_extens_status().is_some());

        assert!(wall.is_wall());
        assert!(candidate.is_candidate_wall());
    }
}
