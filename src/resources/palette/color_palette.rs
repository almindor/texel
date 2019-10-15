use serde::{Deserialize, Serialize};
use crate::common::LazyLoaded;

const fn cc(r: u8, g: u8, b: u8) -> u8 {
    16 + 36 * r + 6 * g + b
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    fn default() -> ColorPalette {
        ColorPalette {
            colors: DEFAULT_PALETTE_COLORS,
            fg_string: Self::to_line_string(&DEFAULT_PALETTE_COLORS, ColorMode::Fg),
            bg_string: Self::to_line_string(&DEFAULT_PALETTE_COLORS, ColorMode::Bg),
        }
    }
}

impl From<[u8; COLORS_IN_PALETTE]> for ColorPalette {
    fn from(colors: [u8; COLORS_IN_PALETTE]) -> Self {
        ColorPalette {
            colors,
            fg_string: Self::to_line_string(&colors, ColorMode::Fg),
            bg_string: Self::to_line_string(&colors, ColorMode::Bg),
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
            fg_string: Self::to_line_string(&palette_colors, ColorMode::Fg),
            bg_string: Self::to_line_string(&palette_colors, ColorMode::Bg),
        }
    }
}

impl LazyLoaded for ColorPalette {
    fn refresh(&mut self) {
        if self.fg_string.is_empty() {
            self.fg_string = Self::to_line_string(&self.colors, ColorMode::Fg);
        }

        if self.bg_string.is_empty() {
            self.bg_string = Self::to_line_string(&self.colors, ColorMode::Bg);
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

    fn re_rgb(color: u8) -> (u8, u8, u8) {
        let base = color - 16;
        (base / 36, (base / 6) % 6, base % 6)
    }

    fn luminance(color: u8) -> u8 {
        let (r, g, b) = Self::re_rgb(color);
        // get luminance according to spec (output is in 0..6 tho same as ansivalue bases)
        let y = (0.2126 * f32::from(r) + 0.7151 * f32::from(g) + 0.0721 * f32::from(b)) as u8;
        y
    }

    fn invert_fg(color: u8) -> String {
        if Self::luminance(color) > 2 {
            termion::color::AnsiValue::grayscale(5).fg_string()
        } else {
            termion::color::AnsiValue::grayscale(17).fg_string()
        }
    }

    fn invert_bg(color: u8) -> String {
        if Self::luminance(color) > 2 {
            termion::color::AnsiValue::grayscale(5).bg_string()
        } else {
            termion::color::AnsiValue::grayscale(17).bg_string()
        }
    }

    fn u8_to_bg_string(color: u8) -> String {
        Self::u8_to_bg(color) + &Self::invert_fg(color)
    }

    fn u8_to_fg_string(color: u8) -> String {
        Self::invert_bg(color) + &Self::u8_to_fg(color)
    }

    fn to_line_string(pc: &[u8], cm: ColorMode) -> String {
        let mut result = String::with_capacity(COLORS_IN_PALETTE * 20);

        for (i, c) in pc.iter().enumerate() {
            let color_string = match cm {
                ColorMode::Fg => Self::u8_to_fg_string(*c),
                ColorMode::Bg => Self::u8_to_bg_string(*c),
            };

            result += &color_string;
            result.push(COLOR_SELECTOR[i]);
        }

        result
    }

    // lazy load so we can skip serializing but don't need to do custom serde crap
    pub fn line_str(&self, cm: ColorMode) -> &str {
        match cm {
            ColorMode::Fg => &self.fg_string,
            ColorMode::Bg => &self.bg_string,
        }
    }
}