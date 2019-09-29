use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

const fn cc(r: u8, g: u8, b: u8) -> u8 {
    16 + 36 * r + 6 * g + b
}

const COLORS_IN_PALETTE: usize = 16;
const DEFAULT_PALETTE_COLORS: [u8; COLORS_IN_PALETTE] = [
    cc(5, 5, 5), cc(0, 0, 0), // b & w
    cc(5, 0, 0), cc(0, 5, 0), cc(0, 0, 5), // r, g, b
    18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28 // ??
];
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

    pub fn u8_to_bg(color: u8) -> String {
        termion::color::AnsiValue(color).bg_string()
    }

    pub fn invert_fg(color: u8) -> String {
        let base = color - 16;
        let r = base / 36;
        let g = (base / 6) % 6;
        let b = base % 6;
        // get luminance according to spec (output is in 0..6 tho same as ansivalue bases)
        let y = (0.2126 * f32::from(r) + 0.7151 * f32::from(g) + 0.0721 * f32::from(b)) as u8;

        // we don't want to change greyness per each color but only in a range of 
        // 2 options to keep it more consistent to the eye        
        if y > 2 {
            termion::color::AnsiValue::grayscale(3).fg_string()
        } else {
            termion::color::AnsiValue::grayscale(19).fg_string()
        }
    }

    fn to_line_string(pc: &[u8]) -> String {
        let mut result = String::with_capacity(COLORS_IN_PALETTE * 20);

        for (i, c) in pc.iter().enumerate() {
            result += &Self::u8_to_bg(*c);
            result += &Self::invert_fg(*c);
            result.push(COLOR_SELECTOR[i]);
        }

        result
    }

    pub fn line_str(&self) -> &str {
        &self.line_string
    }
}
