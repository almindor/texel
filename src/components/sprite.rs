use crate::common::{cwd_path, Texel};
use crate::resources::{ColorMode, ColorPalette};
use crate::components::Position;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 256 * 256 ascii chars maximum
pub const SPRITE_MAX_BYTES: usize = u16::max_value() as usize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprite {
    pub texels: Vec<Texel>,
}

impl Sprite {
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let abs_path = cwd_path(path)?;

        let mut f = File::open(abs_path)?;
        let mut buf: String = String::with_capacity(SPRITE_MAX_BYTES);
        let byte_size = f.read_to_string(&mut buf)?;

        if byte_size > SPRITE_MAX_BYTES {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }

        let mut texels = Vec::new();

        let mut x = 0;
        let mut y = 0;
        for c in buf.chars() {
            match c {
                ' ' => x += 1,
                '\n' => {
                    x = 0;
                    y += 1;
                }
                _ => {
                    texels.push(Texel {
                        x,
                        y,
                        symbol: c,
                        fg: ColorPalette::default_fg_u8(),
                        bg: ColorPalette::default_bg_u8(),
                    });
                    x += 1;
                }
            }
        }

        Ok(Sprite::from_texels(texels))
    }

    pub fn from_texels(texels: Vec<Texel>) -> Sprite {
        Sprite { texels }
    }

    pub fn fill(&mut self, cm: ColorMode, color: u8) {
        for texel in self.texels.iter_mut() {
            match cm {
                ColorMode::Fg => texel.fg = color,
                ColorMode::Bg => texel.bg = color,
            }
        }
    }

    pub fn apply_symbol(&mut self, symbol: char, bg: u8, fg: u8, pos: Position) -> bool {
        for t in self.texels.iter_mut().filter(|t| {
            t.x == pos.x && t.y == pos.y
        }) {
            if t.symbol == symbol && t.bg == bg && t.fg == fg {
                return false
            }

            t.symbol = symbol;
            t.bg = bg;
            t.fg = fg;

            return true
        }

        self.texels.push(Texel {
            symbol,
            bg,
            fg,
            x: pos.x,
            y: pos.y
        });

        self.recalculate_xy(pos);

        true
    }

    fn recalculate_xy(&mut self, pos: Position) {
        if pos.x < 0 {
            for t in self.texels.iter_mut() {
                t.x -= pos.x
            }
        }

        if pos.y < 0 {
            for t in self.texels.iter_mut() {
                t.y -= pos.y
            }
        }
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}
