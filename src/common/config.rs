use crate::resources::{ColorPalette, SymbolPalette};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Config {
    V1(ConfigV1),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigV1 {
    color_palette: ColorPalette,
    symbol_palette: SymbolPalette,
}
