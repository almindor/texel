use crate::components::Dimension;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// used to keep cursor position etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position2D {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bounds {
    Binding(Position, Dimension),
    Free(Position, Dimension),
}

impl Bounds {
    pub fn position(&self) -> &Position {
        match self {
            Bounds::Binding(p, _) => p,
            Bounds::Free(p, _) => p,
        }
    }

    pub fn dimension(&self) -> &Dimension {
        match self {
            Bounds::Binding(_, d) => d,
            Bounds::Free(_, d) => d,
        }
    }

    pub fn right(&self) -> i32 {
        self.position().x + i32::from(self.dimension().w) - 1
    }

    pub fn bottom(&self) -> i32 {
        self.position().y + i32::from(self.dimension().h) - 1
    }
}

impl Default for Position {
    fn default() -> Self {
        Position { x: 1, y: 1, z: 0 }
    }
}

impl Default for Position2D {
    fn default() -> Self {
        Position2D { x: 1, y: 1 }
    }
}

impl From<&Position> for Position2D {
    fn from(pos: &Position) -> Position2D {
        Position2D {
            x: pos.x,
            y: pos.y,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.z != 0 {
            write!(f, "{},{},{}", self.x, self.y, self.z)
        } else {
            write!(f, "{},{}", self.x, self.y)
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Position;

    fn sub(self, other: Self) -> Self {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Sub for Position2D {
    type Output = Position2D;

    fn sub(self, other: Self) -> Self {
        Position2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Sub<Position> for Position2D {
    type Output = Position2D;

    fn sub(self, other: Position) -> Self {
        Position2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Position {
    pub fn from_xyz(x: i32, y: i32, z: i32) -> Self {
        Position { x, y, z }
    }

    pub fn apply(&mut self, translation: Translation, bounds: Bounds) -> bool {
        match translation {
            Translation::None => {}
            Translation::Relative(x, y, z) => {
                self.x += i32::from(x);
                self.y += i32::from(y);
                self.z += i32::from(z);
            }
            Translation::Absolute(x, y, z) => {
                self.x = x;
                self.y = y;
                if let Some(z) = z {
                    self.z = z;
                }
            }
            Translation::ToEdge(dir) => match dir {
                Direction::Left => self.x = bounds.position().x,
                Direction::Top => self.y = bounds.position().y,
                Direction::Bottom => self.y = bounds.position().y + i32::from(bounds.dimension().h),
                Direction::Right => self.x = bounds.position().x + i32::from(bounds.dimension().w),
            },
        }

        match bounds {
            Bounds::Binding(p, _) => {
                if self.x < p.x {
                    self.x = p.x;
                    false
                } else if self.y < p.y {
                    self.y = p.y;
                    false
                } else if self.x > bounds.right() {
                    self.x = bounds.right();
                    false
                } else if self.y > bounds.bottom() {
                    self.y = bounds.bottom();
                    false
                } else {
                    true
                }
            }
            _ => true,
        }
    }
}

impl Position2D {
    pub fn apply(&mut self, translation: Translation, bounds: Bounds) -> bool {
        let mut pos3d = Position { x: self.x, y: self.y, z: 0 };

        if pos3d.apply(translation, bounds) {
            self.x = pos3d.x;
            self.y = pos3d.y;
            true
        } else {
            false
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Component for Position2D {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Top,
    Bottom,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Translation {
    None,
    Relative(i32, i32, i32),
    Absolute(i32, i32, Option<i32>),
    ToEdge(Direction),
}

impl Default for Translation {
    fn default() -> Self {
        Translation::None
    }
}
