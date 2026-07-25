# maze_cell データ構造 図解まとめ

## 1. 全体構造（型の階層）

```mermaid
classDiagram
direction TB

class MazePoint {
  +x: u64
  +y: u64
  +new(x, y)
  +generate_adjacent_maze_points(distance)
}

class MazePointStatus {
  <<enum>>
  Path(PathType, Enforcer)
  Wall(WallType, Option<WallIdentifier>, Enforcer)
}

class PathType {
  <<enum>>
  StartOrEnd
  ResolvedPath
  NotResolvedPath
}

class WallType {
  <<enum>>
  Outside(OutsideWallType, ExtendStatus, Enforcer)
  Pillar(ExtendStatus, Enforcer)
  MazeWall(Enforcer)
}

class OutsideWallType {
  <<enum>>
  StartPoint
  JustWall
}

class ExtendStatus {
  <<enum>>
  NotChecked
  Extending
}

class WallIdentifier {
  tid_id: ThreadId
  unix_time_nanos: i64
  +new()
}

MazePointStatus --> PathType : Path variant
MazePointStatus --> WallType : Wall variant
MazePointStatus --> WallIdentifier : optional owner
WallType --> OutsideWallType : Outside variant
WallType --> ExtendStatus : state
```

## 2. 役割の分担（要点）

- MazePoint
  - 座標そのものを表す値オブジェクト。
  - 近傍座標の生成や、座標計算の基盤を担当。
- MazePointStatus
  - 1マスの最上位状態。
  - 通路か壁かをまず決める「親の型」。
- PathType
  - 通路の意味付け。
  - 始点/終点、探索済み、未探索を区別。
- WallType
  - 壁の意味付け。
  - 外壁、柱、通常迷路壁を区別。
- OutsideWallType
  - 外壁の中でも「開始点候補」か「ただの境界壁」かを区別。
- ExtendStatus
  - 拡張アルゴリズム上の進行状態。
  - 未処理か、拡張中かを表す。
- WallIdentifier
  - 壁の所有者/生成元を追跡する識別子。
  - マルチスレッド時の由来管理に使う。

## 3. 状態遷移（壁生成の流れ）

```mermaid
stateDiagram-v2
direction LR

[*] --> Path_NotResolved : 初期通路

state "Outside(StartPoint)" as OSP {
  [*] --> NotChecked
  NotChecked --> Extending : 開始点として選択
}

state "Pillar" as P {
  [*] --> NotChecked
  NotChecked --> Extending : 柱の拡張開始
}

state "Outside(JustWall)" as OJ {
  [*] --> FixedNotChecked
}

state "MazeWall" as MW {
  [*] --> Fixed
}

Path_NotResolved --> Path_Resolved : 探索で到達
Path_NotResolved --> Path_StartOrEnd : 始点/終点設定
```

## 4. 実行時イメージ（簡易フロー）

```mermaid
flowchart TD
A[座標 MazePoint を列挙] --> B[各座標に MazePointStatus を割当]
B --> C{壁か通路か}
C -->|通路| D[PathType を設定]
C -->|壁| E[WallType を設定]
E --> F{Outside / Pillar / MazeWall}
F --> G[必要に応じて ExtendStatus を更新]
G --> H[必要に応じて WallIdentifier を付与]
```

## 5. 一言でいうと

- 設計の芯は「1マス状態を MazePointStatus で統一し、その中で通路と壁をさらに型で細分化する」ことです。
- これにより、迷路生成ロジックが型で安全に分岐でき、並列処理時の所有者管理まで表現できる構造になっています。
