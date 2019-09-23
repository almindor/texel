use crate::components::Sprite;
use specs::{Component, VecStorage};
use std::convert::TryInto;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
    pub w: u16,
    pub h: u16,
}

impl Component for Dimension {
    type Storage = VecStorage<Self>;
}

impl Dimension {
    pub fn for_sprite(sprite: &Sprite) -> Result<Self, std::num::TryFromIntError> {
        let mut w = 0i32;
        let mut h = 0i32;

        for t in &sprite.texels {
            if t.x > w {
                w = t.x;
            }
            if t.y > h {
                h = t.y;
            }
        }

        w += 1;
        h += 1;

        Ok(Dimension {
            w: w.try_into()?,
            h: h.try_into()?,
        })
    }
}
