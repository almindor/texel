
use specs::{Component, VecStorage};

pub use crate::common::SpriteV1 as Sprite;

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}
