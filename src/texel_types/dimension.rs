use crate::texel_types::{Position2D, SpriteV1};
use std::convert::TryInto;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
    pub w: u16,
    pub h: u16,
}

impl From<(u16, u16)> for Dimension {
    fn from(source: (u16, u16)) -> Self {
        Dimension {
            w: source.0,
            h: source.1,
        }
    }
}

impl std::ops::Sub for Dimension {
    type Output = Dimension;

    fn sub(self, other: Self) -> Self::Output {
        let w = if other.w > self.w { 0 } else { self.w - other.w };
        let h = if other.h > self.h { 0 } else { self.h - other.h };

        Dimension { w, h }
    }
}

impl Dimension {
    pub fn unit() -> Self {
        Dimension { w: 1, h: 1 }
    }

    pub fn size(self) -> usize {
        usize::from(self.w * self.h)
    }

    pub fn for_area(top_left: Position2D, bottom_right: Position2D) -> Self {
        Dimension {
            w: (bottom_right.x - top_left.x + 1) as u16,
            h: (bottom_right.y - top_left.y + 1) as u16,
        }
    }

    pub fn from_wh(w: u16, h: u16) -> Self {
        Dimension { w, h }
    }

    pub fn for_sprite(sprite: &SpriteV1) -> Self {
        let mut w32 = 0i32;
        let mut h32 = 0i32;

        for t in sprite.all_iter() {
            if t.x > w32 {
                w32 = t.x;
            }
            if t.y > h32 {
                h32 = t.y;
            }
        }

        w32 += 1;
        h32 += 1;

        Dimension {
            w: w32.try_into().unwrap_or_else(|_| 0),
            h: h32.try_into().unwrap_or_else(|_| 0),
        }
    }
}
