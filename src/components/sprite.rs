
use specs::{Component, VecStorage};

pub use crate::texel_types::SpriteV1 as Sprite;

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}
