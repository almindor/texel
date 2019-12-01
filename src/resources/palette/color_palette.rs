use crate::common::{Error, SymbolStyle, SymbolStyles, Texel, Texels};
use crate::components::Position2D;
use serde::{Deserialize, Serialize};

const fn cc(r: u8, g: u8, b: u8) -> u8 {
    16 + 36 * r + 6 * g + b
}

pub const PALETTE_W: i32 = 16;
pub const PALETTE_H: i32 = 14;
pub const PALETTE_OFFSET: i32 = 24;
pub const MAX_COLOR_INDEX: u8 = 6 * 6 * 6;

const COLORS_IN_PALETTE: usize = 16;

const DEFAULT_PALETTE_COLORS: [u8; COLORS_IN_PALETTE] = [
    cc(5, 5, 5),
    cc(0, 0, 0), // b & w
    cc(5, 0, 0),
    cc(0, 5, 0),
    cc(0, 0, 5), // r, g, b
    cc(5, 5, 0),
    cc(0, 5, 5),
    cc(5, 0, 5),
    cc(2, 5, 2),
    cc(5, 2, 2),
    cc(2, 2, 5),
    cc(2, 5, 5),
    cc(5, 5, 2),
    cc(5, 2, 5),
    27,
    28, // ??
];

const COLOR_SELECTOR: [char; COLORS_IN_PALETTE] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ColorMode {
    Bg,
    Fg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    colors: [u8; COLORS_IN_PALETTE],
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            colors: DEFAULT_PALETTE_COLORS,
        }
    }
}

impl From<[u8; COLORS_IN_PALETTE]> for ColorPalette {
    fn from(colors: [u8; COLORS_IN_PALETTE]) -> Self {
        ColorPalette { colors }
    }
}

impl From<&[u8]> for ColorPalette {
    fn from(colors: &[u8]) -> Self {
        let mut palette_colors: [u8; COLORS_IN_PALETTE] = [0; COLORS_IN_PALETTE];

        for (i, color) in colors.iter().enumerate() {
            palette_colors[i] = *color;
        }

        ColorPalette { colors: palette_colors }
    }
}

impl ColorPalette {
    pub fn color(&self, index: usize) -> u8 {
        // index here is natural digit conversion, but we go 1,2...9,0,a,b,c...f
        let mut i = index;
        if index == 0 {
            i = 9;
        } else if index < 10 {
            i = index - 1;
        }

        self.colors[i]
    }

    pub fn set_color(&mut self, index: usize, color: u8) -> Result<(), Error> {
        if index >= self.colors.len() {
            return Err(Error::execution("Index out of bounds"));
        }

        self.colors[index] = color;

        Ok(())
    }

    pub fn default_fg_u8() -> u8 {
        termion::color::AnsiValue::grayscale(16).0
    }

    pub fn default_bg_u8() -> u8 {
        termion::color::AnsiValue::rgb(0, 0, 0).0
    }

    pub fn subselection_bg_u8() -> u8 {
        termion::color::AnsiValue::grayscale(14).0
    }

    pub fn u8_to_fg(color: u8) -> String {
        termion::color::AnsiValue(color).fg_string()
    }

    pub fn u8_to_bg(color: u8) -> String {
        termion::color::AnsiValue(color).bg_string()
    }

    pub fn pos_to_color(pos: Position2D) -> u8 {
        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let min = Position2D {
            x: PALETTE_OFFSET,
            y: i32::from(ts.1) - PALETTE_H,
        };
        let mut base = Self::pos_to_base(pos - min);

        if base >= MAX_COLOR_INDEX {
            base = 0; // black on black
        }

        base + 16
    }

    pub const fn base_to_rgb(base: u8) -> (u8, u8, u8) {
        (base / 36, (base / 6) % 6, base % 6)
    }

    pub const fn pos_to_base(pos: Position2D) -> u8 {
        (pos.y * PALETTE_W) as u8 + pos.x as u8
    }

    pub fn selector_texel(&self, index: usize, x: i32, y: i32, cm: ColorMode) -> Texel {
        let (bg, fg) = match cm {
            ColorMode::Bg => (self.colors[index], invert_luminance(self.colors[index])),
            ColorMode::Fg => (invert_luminance(self.colors[index]), self.colors[index]),
        };
        let s_u8 = crate::common::index_from_one(index) as u8;

        Texel {
            x,
            y,
            fg,
            bg,
            symbol: char::from(s_u8),
            styles: SymbolStyles::only(SymbolStyle::Bold),
        }
    }

    pub fn line_texels(&self, start_x: i32, y: i32, cm: ColorMode) -> Texels {
        let mut result = Vec::with_capacity(COLORS_IN_PALETTE);

        let mut x = start_x;
        for (i, symbol) in COLOR_SELECTOR.iter().enumerate() {
            let (bg, fg) = match cm {
                ColorMode::Bg => (self.colors[i], invert_luminance(self.colors[i])),
                ColorMode::Fg => (invert_luminance(self.colors[i]), self.colors[i]),
            };

            result.push(Texel {
                x,
                y,
                fg,
                bg,
                symbol: *symbol,
                styles: SymbolStyles::only(SymbolStyle::Bold),
            });
            x += 1;
        }

        result
    }
}

fn re_rgb(color: u8) -> (u8, u8, u8) {
    let base = color - 16;
    (base / 36, (base / 6) % 6, base % 6)
}

fn luminance(color: u8) -> u8 {
    let (r, g, b) = re_rgb(color);
    // get luminance according to spec (output is in 0..6 tho same as ansivalue bases)
    (0.2126 * f32::from(r) + 0.7151 * f32::from(g) + 0.0721 * f32::from(b)) as u8
}

fn invert_luminance(color: u8) -> u8 {
    if luminance(color) > 2 {
        termion::color::AnsiValue::grayscale(5).0
    } else {
        termion::color::AnsiValue::grayscale(17).0
    }
}
