use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    colors: [u8; 16],
}

impl Default for ColorPalette {
    fn default() -> ColorPalette {
        ColorPalette {
            colors: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        }
    }
}

impl From<[u8; 16]> for ColorPalette {
    fn from(colors: [u8; 16]) -> Self {
        ColorPalette { colors }
    }
}

impl From<&[u8]> for ColorPalette {
    fn from(colors: &[u8]) -> Self {
        let mut palette_colors: [u8; 16] = [0; 16];

        for (i, color) in colors.iter().enumerate() {
            palette_colors[i] = *color;
        }

        ColorPalette {
            colors: palette_colors,
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

    pub fn default_bg() -> &'static str {
        termion::color::Reset.bg_str()
    }

    pub fn u8_to_fg(color: u8) -> String {
        termion::color::AnsiValue(color).fg_string()
    }

    pub fn u8_to_bg(color: u8) -> String {
        termion::color::AnsiValue(color).bg_string()
    }
}
