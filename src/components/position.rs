use specs::{Component, VecStorage};

pub use crate::texel_types::{Position, Position2D, Bounds, Translation, Direction};

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.z != 0 {
            write!(f, "{},{},{}", self.x, self.y, self.z)
        } else {
            write!(f, "{},{}", self.x, self.y)
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Component for Position2D {
    type Storage = VecStorage<Self>;
}

