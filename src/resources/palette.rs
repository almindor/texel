use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

const COLORS_IN_PALETTE: usize = 16;
const DEFAULT_PALETTE_COLORS: [u8; COLORS_IN_PALETTE] =
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const COLOR_SELECTOR: [char; COLORS_IN_PALETTE] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    colors: [u8; COLORS_IN_PALETTE],
    line_string: String,
}

impl Default for ColorPalette {
    fn default() -> ColorPalette {
        ColorPalette {
            colors: DEFAULT_PALETTE_COLORS,
            line_string: Self::to_line_string(&DEFAULT_PALETTE_COLORS),
        }
    }
}

impl From<[u8; COLORS_IN_PALETTE]> for ColorPalette {
    fn from(colors: [u8; COLORS_IN_PALETTE]) -> Self {
        ColorPalette {
            colors,
            line_string: Self::to_line_string(&colors),
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
            line_string: Self::to_line_string(&palette_colors),
        }
    }
}

impl Component for ColorPalette {
    type Storage = VecStorage<Self>;
}

impl ColorPalette {
    pub fn default_fg() -> &'static str {
        termion::color::Reset.fg_str()
    }

    // pub fn default_bg() -> &'static str {
    //     termion::color::Reset.bg_str()
    // }

    pub fn u8_to_fg(color: u8) -> String {
        termion::color::AnsiValue(color).fg_string()
    }

    pub fn u8_to_bg(color: u8) -> String {
        termion::color::AnsiValue(color).bg_string()
    }

    fn to_line_string(pc: &[u8]) -> String {
        let mut result = String::with_capacity(COLORS_IN_PALETTE * 10);
        for (i, c) in pc.iter().enumerate() {
            result = Self::u8_to_bg(*c) + &Self::u8_to_fg(*c);
            result.push(COLOR_SELECTOR[i]);
        }

        result
    }

    pub fn line_str(&self) -> &str {
        &self.line_string
    }
}
