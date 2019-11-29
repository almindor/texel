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
    Binding(Position2D, Dimension),
    Free(Position2D, Dimension),
}

impl std::iter::IntoIterator for Bounds {
    type Item = Position2D;
    type IntoIter = BoundsIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BoundsIntoIterator { bounds: self, index: 0 }
    }
}

pub struct BoundsIntoIterator {
    bounds: Bounds,
    index: usize,
}

impl std::iter::Iterator for BoundsIntoIterator {
    type Item = Position2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bounds.size() {
            return None;
        }

        let old_index = self.index;
        self.index += 1;

        use crate::common::coords_from_index;
        if let Some(pos) = coords_from_index(old_index, *self.bounds.dimension()) {
            Some(pos + *self.bounds.position())
        } else {
            None
        }
    }
}

impl std::ops::Sub<Position2D> for Bounds {
    type Output = Bounds;

    fn sub(self, other: Position2D) -> Self::Output {
        match self {
            Bounds::Binding(pos, dim) => Bounds::Binding(pos - other, dim),
            Bounds::Free(pos, dim) => Bounds::Free(pos - other, dim),
        }
    }
}

impl Bounds {
    pub fn empty() -> Self {
        Bounds::Free(Position2D { x: 0, y: 0 }, Dimension::default())
    }

    pub fn point(pos: Position2D) -> Self {
        Bounds::Binding(pos, Dimension::unit())
    }

    pub fn position(&self) -> &Position2D {
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

    pub fn size(&self) -> usize {
        self.dimension().size()
    }

    pub fn right(&self) -> i32 {
        self.position().x + i32::from(self.dimension().w) - 1
    }

    pub fn bottom(&self) -> i32 {
        self.position().y + i32::from(self.dimension().h) - 1
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        let pos = self.position();
        let dim = self.dimension();

        x >= pos.x && x < pos.x + i32::from(dim.w) && y >= pos.y && y < pos.y + i32::from(dim.h)
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

impl From<&mut Position> for Position2D {
    fn from(pos: &mut Position) -> Position2D {
        Position2D { x: pos.x, y: pos.y }
    }
}

impl From<&Position> for Position2D {
    fn from(pos: &Position) -> Position2D {
        Position2D { x: pos.x, y: pos.y }
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

impl std::ops::Add<Position2D> for Position2D {
    type Output = Position2D;

    fn add(self, other: Position2D) -> Self::Output {
        Position2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Add<Position2D> for Position {
    type Output = Position;

    fn add(self, other: Position2D) -> Self::Output {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z,
        }
    }
}

impl std::ops::AddAssign<Position2D> for Position {
    fn add_assign(&mut self, other: Position2D) {
        self.x += other.x;
        self.y += other.y;
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
    pub fn apply(&mut self, translation: Translation, bounds: Bounds) -> bool {
        match translation {
            Translation::None => {}
            Translation::Relative(x, y, z) => {
                self.x += x;
                self.y += y;
                self.z += z;
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
                Direction::Bottom => self.y = bounds.bottom(),
                Direction::Right => self.x = bounds.right(),
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
        let mut pos3d = Position {
            x: self.x,
            y: self.y,
            z: 0,
        };

        if pos3d.apply(translation, bounds) {
            self.x = pos3d.x;
            self.y = pos3d.y;
            true
        } else {
            false
        }
    }

    // create bounds from two points
    pub fn area(self, other: Position2D) -> Bounds {
        let top_left = Position2D {
            x: std::cmp::min(self.x, other.x),
            y: std::cmp::min(self.y, other.y),
        };
        let bottom_right = Position2D {
            x: std::cmp::max(self.x, other.x),
            y: std::cmp::max(self.y, other.y),
        };

        let dim = Dimension::for_area(top_left, bottom_right);

        Bounds::Binding(top_left, dim)
    }

    // "create" the list of all positions in given area from self -> dim
    pub fn area_texels(self, dim: Dimension) -> Vec<Position2D> {
        let mut result = Vec::with_capacity(dim.size());

        for x in self.x..self.x + i32::from(dim.w) {
            for y in self.y..self.y + i32::from(dim.h) {
                result.push(Position2D { x, y });
            }
        }

        result
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
