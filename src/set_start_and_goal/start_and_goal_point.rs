use crate::maze::maze_cell::maze_point::point::MazePoint;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StartAndGoalPoint {
    pub start: MazePoint,
    pub goal: MazePoint,
}
