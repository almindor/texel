use crate::common::{Error, LazyLoaded};
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
    #[serde(skip_serializing)]
    #[serde(default)]
    fg_string: String,
    #[serde(skip_serializing)]
    #[serde(default)]
    bg_string: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            colors: DEFAULT_PALETTE_COLORS,
            fg_string: to_line_string(&DEFAULT_PALETTE_COLORS, ColorMode::Fg),
            bg_string: to_line_string(&DEFAULT_PALETTE_COLORS, ColorMode::Bg),
        }
    }
}

impl From<[u8; COLORS_IN_PALETTE]> for ColorPalette {
    fn from(colors: [u8; COLORS_IN_PALETTE]) -> Self {
        ColorPalette {
            colors,
            fg_string: to_line_string(&colors, ColorMode::Fg),
            bg_string: to_line_string(&colors, ColorMode::Bg),
        }
    }
}

impl From<&[u8]> for ColorPalette {
    fn from(colors: &[u8]) -> Self {
        let mut palette_colors: [u8; COLORS_IN_PALETTE] = [0; COLORS_IN_PALETTE];

        for (i, color) in colors.iter().enumerate() {
            palette_colors[i] = *color;
        }

        ColorPalette {
            colors: palette_colors,
            fg_string: to_line_string(&palette_colors, ColorMode::Fg),
            bg_string: to_line_string(&palette_colors, ColorMode::Bg),
        }
    }
}

impl LazyLoaded for ColorPalette {
    fn refresh(&mut self) {
        if self.fg_string.is_empty() {
            self.fg_string = to_line_string(&self.colors, ColorMode::Fg);
        }

        if self.bg_string.is_empty() {
            self.bg_string = to_line_string(&self.colors, ColorMode::Bg);
        }
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
        self.bg_string = to_line_string(&self.colors, ColorMode::Bg);
        self.fg_string = to_line_string(&self.colors, ColorMode::Fg);

        Ok(())
    }

    pub fn default_fg_u8() -> u8 {
        termion::color::AnsiValue::grayscale(16).0
    }

    pub fn default_bg_u8() -> u8 {
        termion::color::AnsiValue::grayscale(0).0
    }

    pub fn default_fg() -> &'static str {
        termion::color::Reset.fg_str()
    }

    pub fn default_bg() -> &'static str {
        termion::color::Reset.bg_str()
    }

    pub fn u8_to_fg(color: u8) -> String {
        termion::color::AnsiValue(color).fg_string()
    }

    pub fn u8_to_bg(color: u8) -> String {
        termion::color::AnsiValue(color).bg_string()
    }

    pub fn u8_to_bg_string(color: u8) -> String {
        ColorPalette::u8_to_bg(color) + &invert_fg(color)
    }

    pub fn u8_to_fg_string(color: u8) -> String {
        invert_bg(color) + &ColorPalette::u8_to_fg(color)
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

    // lazy load so we can skip serializing but don't need to do custom serde crap
    pub fn line_str(&self, cm: ColorMode) -> &str {
        match cm {
            ColorMode::Fg => &self.fg_string,
            ColorMode::Bg => &self.bg_string,
        }
    }
}

fn re_rgb(color: u8) -> (u8, u8, u8) {
    let base = color - 16;
    (base / 36, (base / 6) % 6, base % 6)
}

fn luminance(color: u8) -> u8 {
    let (r, g, b) = re_rgb(color);
    // get luminance according to spec (output is in 0..6 tho same as ansivalue bases)
    let y = (0.2126 * f32::from(r) + 0.7151 * f32::from(g) + 0.0721 * f32::from(b)) as u8;
    y
}

fn invert_fg(color: u8) -> String {
    if luminance(color) > 2 {
        termion::color::AnsiValue::grayscale(5).fg_string()
    } else {
        termion::color::AnsiValue::grayscale(17).fg_string()
    }
}

fn invert_bg(color: u8) -> String {
    if luminance(color) > 2 {
        termion::color::AnsiValue::grayscale(5).bg_string()
    } else {
        termion::color::AnsiValue::grayscale(17).bg_string()
    }
}

fn to_line_string(pc: &[u8], cm: ColorMode) -> String {
    let mut result = String::with_capacity(COLORS_IN_PALETTE * 20);

    for (i, c) in pc.iter().enumerate() {
        let color_string = match cm {
            ColorMode::Fg => ColorPalette::u8_to_fg_string(*c),
            ColorMode::Bg => ColorPalette::u8_to_bg_string(*c),
        };

        result += &color_string;
        result.push(COLOR_SELECTOR[i]);
    }

    result
}
