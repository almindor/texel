pub use texel_types::{Bounds, Sprite, DEFAULT_BG_U8};

// extra stuff for texel only
pub trait SpriteExt {
    fn clear_blank_texels(&mut self, area: Option<Bounds>) -> bool;
}

impl SpriteExt for Sprite {
    fn clear_blank_texels(&mut self, area: Option<Bounds>) -> bool {
        let count = self.frames[self.index].len();
        // remove "empty" (char ' ') texels with default BG in given area
        self.frames[self.index].retain(|t| {
            let mut r = t.symbol != ' ' || t.bg != DEFAULT_BG_U8;
            if let Some(bounds) = area {
                r |= !bounds.contains(t.pos);
            }

            r
        });

        self.frames[self.index].len() != count
    }
}
