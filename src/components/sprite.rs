use crate::common::{cwd_path, Error, SymbolStyle, SymbolStyles, Texel, Texels, Which};
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
    frames: Vec<Texels>, // subsequent vectors are "overrides" from first frame
    index: usize,
}

impl Sprite {
    pub fn frame_index(&self) -> usize {
        self.index
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn new_frame(&mut self) {
        self.frames.push(self.frames[self.index].clone());
        self.apply_frame_change(Which::Next);
    }

    pub fn delete_frame(&mut self) -> bool {
        if self.frames.len() > 1 {
            self.frames.remove(self.index);
            self.apply_frame_change(Which::Previous);
            true
        } else {
            false
        }
    }

    pub fn apply_frame_change(&mut self, which: Which<usize>) -> usize {
        match which {
            Which::Next => self
                .set_frame(self.index + 1)
                .unwrap_or_else(|_| std::cmp::max(self.frames.len(), 1) - 1),
            Which::Previous => self.set_frame(std::cmp::max(self.index, 1) - 1).unwrap_or_else(|_| 0),
            Which::At(index) => self.set_frame(index).unwrap_or_else(|_| self.index),
        }
    }

    fn set_frame(&mut self, index: usize) -> Result<usize, Error> {
        self.index = if index >= self.frames.len() {
            return Err(Error::execution("Invalid frame index"));
        } else {
            index
        };

        Ok(self.index)
    }

    pub fn all_iter(&self) -> impl Iterator<Item = &Texel> {
        self.frames.iter().flatten()
    }

    pub fn all_iter_mut(&mut self) -> impl Iterator<Item = &mut Texel> {
        self.frames.iter_mut().flatten()
    }

    pub fn frame_iter(&self) -> impl Iterator<Item = &Texel> {
        self.frames[self.index].iter()
    }

    pub fn frame_iter_mut(&mut self) -> impl Iterator<Item = &mut Texel> {
        self.frames[self.index].iter_mut()
    }

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

    pub fn from_texels(texels: Texels) -> Sprite {
        Sprite {
            frames: vec![texels],
            index: 0,
        }
    }

    pub fn fill_color(&mut self, cm: ColorMode, color: u8) -> bool {
        let mut changed = false;

        for texel in self.frame_iter_mut() {
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
        for texel in self.frame_iter_mut() {
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
        self.frames[self.index].retain(|t| !area.contains(t.x, t.y));

        // re-add them with new setup
        for pos in area.into_iter() {
            self.frames[self.index].push(Texel {
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

        for t in self.frame_iter_mut().filter(|t| area.contains(t.x, t.y)) {
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

        for t in self.frame_iter_mut().filter(|t| area.contains(t.x, t.y)) {
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
        let count = self.frames[self.index].len();
        self.frames[self.index].retain(|t| !area.contains(t.x, t.y));

        if count != self.frames[self.index].len() {
            return Ok(Some(self.calculate_bounds()?));
        }

        Ok(None)
    }

    fn is_empty(&self) -> bool {
        self.frames.is_empty()
            || self
                .frames
                .iter()
                .map(|inner| inner.is_empty())
                .max()
                .unwrap_or_else(|| false)
    }

    // goes through texels so we can calculate dimension and move position if
    // needed. TODO: optimize, we're doing 3 loops here for no good reason
    fn calculate_bounds(&mut self) -> Result<Bounds, Error> {
        if self.is_empty() {
            return Ok(Bounds::empty());
        }

        let mut min_x = i32::max_value();
        let mut min_y = i32::max_value();

        // get new top/left
        for t in self.all_iter() {
            if t.x < min_x {
                min_x = t.x;
            }
            if t.y < min_y {
                min_y = t.y;
            }
        }

        // shift texels as needed
        if min_x != 0 || min_y != 0 {
            for t in self.all_iter_mut() {
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
