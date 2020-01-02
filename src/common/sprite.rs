pub use texel_types::{Bounds, Position, Sprite, DEFAULT_BG_U8};

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

// meta info for showing in the bottom right corner
// use for multiples too
#[derive(Debug, Clone, Copy)]
pub struct SelectedInfo {
    pub selected_count: usize,
    pub frame_index: usize,
    pub frame_count: usize,
    pub pos: Position, // top left position
}

impl Default for SelectedInfo {
    fn default() -> Self {
        SelectedInfo {
            selected_count: 0,
            frame_index: 0,
            frame_count: 0,
            pos: Position {
                x: i32::max_value(),
                y: i32::max_value(),
                z: 0,
            },
        }
    }
}

impl std::fmt::Display for SelectedInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})::[{}/{}]", self.pos, self.frame_index, self.frame_count,)
    }
}

impl SelectedInfo {
    pub fn append(&mut self, sprite: &Sprite, pos: &Position) {
        self.selected_count += 1;
        self.frame_index = sprite.frame_index();
        self.frame_count = sprite.frame_count();

        if pos.x < self.pos.x {
            self.pos.x = pos.x;
        }

        if pos.y < self.pos.y {
            self.pos.y = pos.y;
        }

        self.pos.z = pos.z;
        if self.selected_count > 1 {
            self.pos.z = 0; // don't show
        }
    }
}
