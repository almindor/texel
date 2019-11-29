use crate::common::{cwd_path, Error, SymbolStyle, SymbolStyles, Texel};
use crate::components::{Bounds, Dimension, Position2D};
use crate::resources::{ColorMode, ColorPalette};
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
                        styles: SymbolStyles::new(),
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

    pub fn fill_color(&mut self, cm: ColorMode, color: u8) -> bool {
        let mut changed = false;

        for texel in self.texels.iter_mut() {
            match cm {
                ColorMode::Fg => {
                    if texel.fg != color {
                        texel.fg = color;
                        changed = true;
                    }
                }
                ColorMode::Bg => {
                    if texel.bg != color {
                        texel.bg = color;
                        changed = true;
                    }
                }
            }
        }

        changed
    }

    pub fn fill_style(&mut self, style: SymbolStyle) -> bool {
        let mut changed = false;
        for texel in self.texels.iter_mut() {
            if texel.styles.contains(style) {
                texel.styles.remove(style);
            } else {
                texel.styles.insert(style);
            }
            changed = true;
        }

        changed
    }

    pub fn apply_symbol(&mut self, symbol: char, bg: u8, fg: u8, area: Bounds) -> Result<Bounds, Error> {
        // remove texels in bounds
        self.texels.retain(|t| !area.contains(t.x, t.y));

        // re-add them with new setup
        for pos in area.into_iter() {
            self.texels.push(Texel {
                symbol,
                bg,
                fg,
                x: pos.x,
                y: pos.y,
                styles: SymbolStyles::new(),
            });
        }

        Ok(self.calculate_bounds()?)
    }

    // TODO: handle empty symbols with BG colors!
    pub fn apply_color(&mut self, cm: ColorMode, color: u8, area: Bounds) -> bool {
        let mut changed = false;

        for t in self.texels.iter_mut().filter(|t| area.contains(t.x, t.y)) {
            if (cm == ColorMode::Bg && t.bg == color) || (cm == ColorMode::Fg && t.fg == color) {
                continue;
            }

            match cm {
                ColorMode::Bg => t.bg = color,
                ColorMode::Fg => t.fg = color,
            }

            changed = true;
        }

        changed
    }

    pub fn apply_style(&mut self, style: SymbolStyle, area: Bounds) -> bool {
        let mut changed = false;

        for t in self.texels.iter_mut().filter(|t| area.contains(t.x, t.y)) {
            if t.styles.contains(style) {
                t.styles.remove(style);
            } else {
                t.styles.insert(style);
            }

            changed = true;
        }

        changed
    }

    pub fn clear_symbol(&mut self, area: Bounds) -> Result<Option<Bounds>, Error> {
        let count = self.texels.len();
        self.texels.retain(|t| !area.contains(t.x, t.y));

        if count != self.texels.len() {
            return Ok(Some(self.calculate_bounds()?));
        }

        Ok(None)
    }

    // goes through texels so we can calculate dimension and move position if
    // needed. TODO: optimize, we're doing 3 loops here for no good reason
    fn calculate_bounds(&mut self) -> Result<Bounds, Error> {
        if self.texels.is_empty() {
            return Ok(Bounds::empty());
        }

        let mut min_x = i32::max_value();
        let mut min_y = i32::max_value();

        // get new top/left
        for t in &self.texels {
            if t.x < min_x {
                min_x = t.x;
            }
            if t.y < min_y {
                min_y = t.y;
            }
        }

        // shift texels as needed
        if min_x != 0 || min_y != 0 {
            for t in self.texels.iter_mut() {
                if min_x != 0 {
                    t.x -= min_x;
                }
                if min_y != 0 {
                    t.y -= min_y;
                }
            }
        }

        Ok(Bounds::Free(
            Position2D { x: min_x, y: min_y },
            Dimension::for_sprite(self)?,
        ))
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}
