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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WallType {
    /// 外壁
    Outside,
    /// 柱
    Pillar(PillarExtendStatus),
    /// 生成された壁
    Wall,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MazePointStatus {
    /// 通路
    Path,
    /// 壁
    Wall(WallType, Option<WallIdentifier>),
}

impl MazePointStatus {
    pub fn new_outside_wall(identifier: WallIdentifier) -> Self {
        MazePointStatus::Wall(WallType::Outside, Some(identifier))
    }

    pub fn new_notchecked_pillar() -> Self {
        MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_point_status_path() {
        let status = MazePointStatus::Path;
        assert!(matches!(status, MazePointStatus::Path));
        assert!(!matches!(status, MazePointStatus::Wall(_, _)));
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

        assert!(matches!(status, MazePointStatus::Wall(_, _)));
        assert!(!matches!(status, MazePointStatus::Path));

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
    fn test_pillar_extend_status_direct_matching() {
        let not_checked = PillarExtendStatus::NotChecked;
        let in_progress = PillarExtendStatus::InProgress;
        let extended = PillarExtendStatus::Extended;

        // 直接パターンマッチングでテスト
        assert!(matches!(not_checked, PillarExtendStatus::NotChecked));
        assert!(!matches!(not_checked, PillarExtendStatus::InProgress));
        assert!(!matches!(not_checked, PillarExtendStatus::Extended));

        assert!(!matches!(in_progress, PillarExtendStatus::NotChecked));
        assert!(matches!(in_progress, PillarExtendStatus::InProgress));
        assert!(!matches!(in_progress, PillarExtendStatus::Extended));

        assert!(!matches!(extended, PillarExtendStatus::NotChecked));
        assert!(!matches!(extended, PillarExtendStatus::InProgress));
        assert!(matches!(extended, PillarExtendStatus::Extended));
    }

    #[test]
    fn test_wall_type_direct_matching() {
        let outside = WallType::Outside;
        let pillar = WallType::Pillar(PillarExtendStatus::NotChecked);
        let wall = WallType::Wall;

        // 直接パターンマッチングでテスト
        assert!(matches!(outside, WallType::Outside));
        assert!(!matches!(outside, WallType::Pillar(_)));
        assert!(!matches!(outside, WallType::Wall));

        assert!(matches!(pillar, WallType::Pillar(_)));
        assert!(!matches!(pillar, WallType::Outside));
        assert!(!matches!(pillar, WallType::Wall));

        // Pillarの内部状態もテスト
        if let WallType::Pillar(status) = pillar {
            assert!(matches!(status, PillarExtendStatus::NotChecked));
        }

        assert!(matches!(wall, WallType::Wall));
        assert!(!matches!(wall, WallType::Pillar(_)));
        assert!(!matches!(wall, WallType::Outside));
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
    fn test_pillar_status_transitions() {
        let identifier = WallIdentifier::new();

        // NotChecked → InProgress
        let not_checked =
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None);
        let in_progress = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::InProgress),
            Some(identifier.clone()),
        );

        // InProgress → Extended
        let extended = MazePointStatus::Wall(
            WallType::Pillar(PillarExtendStatus::Extended),
            Some(identifier.clone()),
        );

        // 各状態の確認
        match not_checked {
            MazePointStatus::Wall(WallType::Pillar(status), _) => {
                assert!(matches!(status, PillarExtendStatus::NotChecked));
            }
            _ => panic!("Expected NotChecked pillar"),
        }

        match in_progress {
            MazePointStatus::Wall(WallType::Pillar(status), _) => {
                assert!(matches!(status, PillarExtendStatus::InProgress));
            }
            _ => panic!("Expected InProgress pillar"),
        }

        match extended {
            MazePointStatus::Wall(WallType::Pillar(status), _) => {
                assert!(matches!(status, PillarExtendStatus::Extended));
            }
            _ => panic!("Expected Extended pillar"),
        }
    }

    #[test]
    fn test_all_wall_types() {
        let outside = WallType::Outside;
        let pillar = WallType::Pillar(PillarExtendStatus::NotChecked);
        let wall = WallType::Wall;

        // Outside型のテスト
        assert!(matches!(outside, WallType::Outside));
        assert!(!matches!(outside, WallType::Pillar(_)));
        assert!(!matches!(outside, WallType::Wall));

        // Pillar型のテスト
        assert!(!matches!(pillar, WallType::Outside));
        assert!(matches!(pillar, WallType::Pillar(_)));
        assert!(!matches!(pillar, WallType::Wall));

        // Wall型のテスト
        assert!(!matches!(wall, WallType::Outside));
        assert!(!matches!(wall, WallType::Pillar(_)));
        assert!(matches!(wall, WallType::Wall));
    }

    #[test]
    fn test_maze_point_status_wall_variants() {
        let identifier = WallIdentifier::new();

        // 各種壁タイプのMazePointStatusをテスト
        let outside_wall = MazePointStatus::Wall(WallType::Outside, Some(identifier.clone()));
        let pillar_wall =
            MazePointStatus::Wall(WallType::Pillar(PillarExtendStatus::NotChecked), None);
        let normal_wall = MazePointStatus::Wall(WallType::Wall, Some(identifier.clone()));

        // 全てwall判定されることを確認
        assert!(matches!(outside_wall, MazePointStatus::Wall(_, _)));
        assert!(matches!(pillar_wall, MazePointStatus::Wall(_, _)));
        assert!(matches!(normal_wall, MazePointStatus::Wall(_, _)));

        // 全てpath判定されないことを確認
        assert!(!matches!(outside_wall, MazePointStatus::Path));
        assert!(!matches!(pillar_wall, MazePointStatus::Path));
        assert!(!matches!(normal_wall, MazePointStatus::Path));

        // 各wall_typeが正しく取得できることを確認
        assert!(matches!(
            outside_wall,
            MazePointStatus::Wall(WallType::Outside, _)
        ));
        assert!(matches!(
            pillar_wall,
            MazePointStatus::Wall(WallType::Pillar(_), _)
        ));
        assert!(matches!(
            normal_wall,
            MazePointStatus::Wall(WallType::Wall, _)
        ));
    }

    #[test]
    fn test_wall_identifier_format() {
        let identifier = WallIdentifier::new();
        let id_str = identifier.as_str();

        // 形式の確認: "ThreadId(...)" + "_" + 数字
        assert!(id_str.starts_with("ThreadId("));
        assert!(id_str.contains("_"));

        // アンダースコア以降が数字であることを確認
        if let Some(pos) = id_str.rfind('_') {
            let time_part = &id_str[pos + 1..];
            assert!(
                time_part.parse::<i64>().is_ok(),
                "Time part should be a valid number"
            );
        } else {
            panic!("Identifier should contain underscore");
        }
    }

    #[test]
    fn test_maze_point_status_matches() {
        let path = MazePointStatus::Path;
        let identifier = WallIdentifier::new();
        let wall = MazePointStatus::Wall(WallType::Outside, Some(identifier));

        // Path状態のテスト
        match path {
            MazePointStatus::Path => {
                // 正常
            }
            MazePointStatus::Wall(_, _) => {
                panic!("Expected Path status");
            }
        }

        // Wall状態のテスト
        match wall {
            MazePointStatus::Path => {
                panic!("Expected Wall status");
            }
            MazePointStatus::Wall(wall_type, id) => {
                assert!(matches!(wall_type, WallType::Outside));
                assert!(id.is_some());
            }
        }
    }
}
