use specs::{Component, VecStorage};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Default for Position {
    fn default() -> Self {
        Position { x: 1, y: 1 }
    }
}

impl Position {
    pub fn from_xy(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn apply(&mut self, translation: Translation, w: u16, h: u16) -> bool {
        match translation {
            Translation::None => {}
            Translation::Relative(x, y) => {
                self.x += i32::from(x);
                self.y += i32::from(y);
            }
            Translation::Absolute(x, y) => {
                self.x = x;
                self.y = y;
            }
            Translation::ToEdge(dir) => match dir {
                Direction::Left(x) => self.x = i32::from(x),
                Direction::Top(y) => self.y = i32::from(y),
                Direction::Bottom(y) => self.y = i32::from(y - h),
                Direction::Right(x) => self.x = i32::from(x - w),
            },
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
    Relative(i32, i32),
    Absolute(i32, i32),
    ToEdge(Direction),
}

impl Default for Translation {
    fn default() -> Self {
        Translation::None
    }
}