pub use texel_types::{Bounds, Dimension, Direction, Position, Position2D, Sprite, Translation};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Selection;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Subselection {
    pub active: bool, // if we're resizing atm.
    pub initial_pos: Position2D,
}

impl Subselection {
    pub fn at(initial_pos: Position2D) -> Self {
        Subselection {
            active: true,
            initial_pos,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Selectable;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Bookmark(pub usize);
