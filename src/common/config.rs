use crate::resources::{ColorPalette, SymbolPalette};
use crate::common::LazyLoaded;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Config {
    V1(ConfigV1),
}

impl Default for Config {
    fn default() -> Self {
        Config::V1(ConfigV1::default())
    }
}

impl Config {
    pub fn current(self) -> ConfigV1 {
        match self {
            Self::V1(mut config) => {
                config.refresh();
                config
            },
            // TODO: once we have V2+ we'll need to return that and convert previous
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigV1 {
    pub color_palette: ColorPalette,
    pub symbol_palette: SymbolPalette,
}

impl From<(&ColorPalette, &SymbolPalette)> for ConfigV1 {
    fn from(palettes: (&ColorPalette, &SymbolPalette)) -> Self {
        ConfigV1 {
            color_palette: palettes.0.clone(),
            symbol_palette: palettes.1.clone(),
        }
    }
}

impl LazyLoaded for ConfigV1 {
    fn refresh(&mut self) {
        self.color_palette.refresh();
        self.symbol_palette.refresh();
    }
}