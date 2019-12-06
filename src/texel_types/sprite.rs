use crate::texel_types::{Position2D, Dimension, SymbolStyle, SymbolStyles, TexelV1, ColorMode, Bounds};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

// default texel colors for sprites
pub const DEFAULT_BG_U8: u8 = 16;
pub const DEFAULT_FG_U8: u8 = 0xE8 + 16;

/// 256 * 256 ascii chars maximum
pub const SPRITE_MAX_BYTES: usize = u16::max_value() as usize;

type Texels = Vec<TexelV1>;

// generic "which" selector for selections etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Which<P> {
    Next,
    Previous,
    At(P), // at index, position or any absolute selector
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteV1 {
    pub frames: Vec<Texels>,
    pub index: usize,
}

impl Default for SpriteV1 {
    fn default() -> Self {
        SpriteV1 {
            frames: vec!(Texels::new()),
            index: 0,
        }
    }
}

impl IntoIterator for SpriteV1 {
    type Item = TexelV1;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    // turn sprite into active frame contents
    fn into_iter(mut self) -> Self::IntoIter {
        if self.frames.is_empty() {
            Vec::new().into_iter()
        } else {
            self.frames.remove(self.index).into_iter()
        }
    }
}

impl SpriteV1 {
    pub fn frame_index(&self) -> usize {
        self.index
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn new_frame(&mut self) {
        self.frames.insert(self.index, self.frames[self.index].clone());
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

    fn set_frame(&mut self, index: usize) -> Result<usize, ()> {
        self.index = if index >= self.frames.len() {
            return Err(());
        } else {
            index
        };

        Ok(self.index)
    }

    pub fn copy_area(&self, area: Bounds) -> Texels {
        let mut result = Texels::new();
        for texel in self.frame_iter().filter(|t| area.contains(t.x, t.y)) {
            result.push(texel.moved_from(*area.position()));
        }

        result
    }

    pub fn all_iter(&self) -> impl Iterator<Item = &TexelV1> {
        self.frames.iter().flatten()
    }

    pub fn all_iter_mut(&mut self) -> impl Iterator<Item = &mut TexelV1> {
        self.frames.iter_mut().flatten()
    }

    pub fn frame_iter(&self) -> impl Iterator<Item = &TexelV1> {
        self.frames[self.index].iter()
    }

    pub fn frame_iter_mut(&mut self) -> impl Iterator<Item = &mut TexelV1> {
        self.frames[self.index].iter_mut()
    }

    pub fn from_txt_file(abs_path: &Path) -> Result<Self, std::io::Error> {
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
                    texels.push(TexelV1 {
                        x,
                        y,
                        symbol: c,
                        styles: SymbolStyles::new(),
                        fg: DEFAULT_FG_U8,
                        bg: DEFAULT_BG_U8,
                    });
                    x += 1;
                }
            }
        }

        Ok(SpriteV1::from_texels(texels))
    }

    pub fn from_texels(texels: Texels) -> SpriteV1 {
        SpriteV1 {
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

    pub fn apply_symbol(&mut self, symbol: char, bg: u8, fg: u8, area: Bounds) -> Bounds {
        // remove texels in bounds
        self.frames[self.index].retain(|t| !area.contains(t.x, t.y));

        // re-add them with new setup
        for pos in area.into_iter() {
            self.frames[self.index].push(TexelV1 {
                symbol,
                bg,
                fg,
                x: pos.x,
                y: pos.y,
                styles: SymbolStyles::new(),
            });
        }

        self.calculate_bounds()
    }

    pub fn apply_texels(&mut self, texels: Texels, pos: Position2D) -> Bounds {
        for texel in texels.into_iter() {
            let localized = texel.move_by(pos);

            if let Some(existing) = self.frames[self.index].iter_mut().find(|t| t.x == localized.x && t.y == localized.y) {
                *existing = localized;
            } else {
                self.frames[self.index].push(localized);
            }
        }

        self.calculate_bounds()
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

    pub fn clear_symbol(&mut self, area: Bounds) -> Option<Bounds> {
        let count = self.frames[self.index].len();
        self.frames[self.index].retain(|t| !area.contains(t.x, t.y));

        if count != self.frames[self.index].len() {
            return Some(self.calculate_bounds());
        }

        None
    }

    pub fn is_empty(&self) -> bool {
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
    fn calculate_bounds(&mut self) -> Bounds {
        if self.is_empty() {
            return Bounds::empty();
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

        Bounds::Free(
            Position2D { x: min_x, y: min_y },
            Dimension::for_sprite(self),
        )
    }
}
