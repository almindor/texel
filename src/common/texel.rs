use crate::resources::ColorPalette;
use big_enum_set::{BigEnumSet, BigEnumSetType};
use serde::{Deserialize, Serialize};

#[derive(Debug, BigEnumSetType, Serialize, Deserialize)]
pub enum SymbolStyle {
    Bold,
    Italic,
    Underline,
}

pub type SymbolStyles = BigEnumSet<SymbolStyle>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Texel {
    pub x: i32,
    pub y: i32,
    pub symbol: char,
    pub styles: BigEnumSet<SymbolStyle>, // u8
    pub fg: u8,
    pub bg: u8,
}

impl std::fmt::Display for Texel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}",
            crate::common::goto(self.x, self.y),
            ColorPalette::u8_to_bg(self.bg),
            ColorPalette::u8_to_fg(self.fg),
            styles_to_str(self.styles),
            self.symbol,
            termion::style::Reset,
        )
    }
}

impl Texel {
    pub fn override_bg(&mut self, bg: u8) {
        self.bg = bg;
    }
}

fn styles_to_str(styles: BigEnumSet<SymbolStyle>) -> String {
    use termion::style::{Bold, Italic, Underline};
    let mut result = String::with_capacity(64);

    for style in styles.iter() {
        result += match style {
            SymbolStyle::Bold => Bold.as_ref(),
            SymbolStyle::Italic => Italic.as_ref(),
            SymbolStyle::Underline => Underline.as_ref(),
        }
    }

    result
}
