mod dimension;
mod position;
mod sprite;

pub use dimension::Dimension;
pub use position::{Direction, Position, Translation};
pub use sprite::Sprite;

// flags

use specs::{Component, NullStorage};

#[derive(Default)]
pub struct Selection;

impl Component for Selection {
    type Storage = NullStorage<Self>;
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
