use specs::{Component, NullStorage, VecStorage};

pub use texel_types::{Bounds, Direction, Position, Position2D, Translation, Dimension, Sprite};

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
