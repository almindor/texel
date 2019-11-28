mod dimension;
mod position;
mod sprite;

pub use dimension::Dimension;
pub use position::{Bounds, Direction, Position, Position2D, Translation};
pub use sprite::Sprite;

// flags

use specs::{Component, NullStorage, VecStorage};

#[derive(Default)]
pub struct Selection;

impl Component for Selection {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
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

impl Component for Subselection {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct Border;

impl Component for Border {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct Selectable;

impl Component for Selectable {
    type Storage = NullStorage<Self>;
}
