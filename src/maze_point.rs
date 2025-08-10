use std::hash::Hash;

///迷路内のXY座標を表す構造体。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct MazePoint {
    x: u32,
    y: u32,
}

impl MazePoint {
    pub fn new(x: u32, y: u32) -> Self {
        MazePoint { x, y }
    }
}

impl MazePoint {
    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }
}
