use crate::components::Dimension;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for Position {
    fn default() -> Self {
        Position { x: 1, y: 1, z: 0 }
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
    }}

impl Position {
    pub fn from_xyz(x: i32, y: i32, z: i32) -> Self {
        Position { x, y, z }
    }

    pub fn apply(
        &mut self,
        translation: Translation,
        dim: &Dimension,
        bounds: Option<(Position, Dimension)>,
    ) -> bool {
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
                Direction::Left(x) => self.x = i32::from(x),
                Direction::Top(y) => self.y = i32::from(y),
                Direction::Bottom(y) => self.y = i32::from(y - dim.h),
                Direction::Right(x) => self.x = i32::from(x - dim.w),
            },
        }

        if let Some(bounds) = bounds {
            if self.x < bounds.0.x {
                self.x = bounds.0.x;
                return false;
            }
            if self.y < bounds.0.y {
                self.y = bounds.0.y;
                return false;
            }
            if self.x > i32::from(bounds.1.w) {
                self.x = i32::from(bounds.1.w);
                return false;
            }
            if self.y > i32::from(bounds.1.h) {
                self.y = i32::from(bounds.1.h);
                return false;
            }
        }

        true
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left(u16),
    Top(u16),
    Bottom(u16),
    Right(u16),
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
