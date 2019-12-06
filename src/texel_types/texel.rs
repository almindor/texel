use crate::texel_types::Position2D;
use big_enum_set::{BigEnumSet, BigEnumSetType};
use serde::{Deserialize, Serialize};

#[derive(Debug, BigEnumSetType, Serialize, Deserialize)]
pub enum SymbolStyle {
    Bold,
    Italic,
    Underline,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ColorMode {
    Bg,
    Fg,
}

pub type SymbolStyles = BigEnumSet<SymbolStyle>;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TexelV1 {
    pub x: i32,
    pub y: i32,
    pub symbol: char,
    pub styles: BigEnumSet<SymbolStyle>, // u8
    pub fg: u8,
    pub bg: u8,
}

impl TexelV1 {
    pub fn moved_from(&self, pos: Position2D) -> Self {
        let mut result = self.clone();

        result.x -= pos.x;
        result.y -= pos.y;

        result
    }

    pub fn move_by(mut self, pos: Position2D) -> Self {
        self.x = self.x + pos.x;
        self.y = self.y + pos.y;

        self
    }

    pub fn override_bg(&mut self, bg: u8) {
        self.bg = bg;
    }
}
