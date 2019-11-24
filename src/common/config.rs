use crate::common::{CharMap, Error};
use crate::resources::{ColorPalette, SymbolPalette};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
            Self::V1(config) => config,
            // TODO: once we have V2+ we'll need to return that and convert previous
        }
    }

    pub fn to_config_file(&self, path: &Path) -> Result<(), Error> {
        let parent = path
            .parent()
            .ok_or_else(|| Error::execution("Unable to create config dif"))?;
        std::fs::create_dir_all(parent)?;

        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, serialized)?;

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigV1 {
    pub color_palette: ColorPalette,
    pub symbol_palette: SymbolPalette,
    pub char_map: CharMap,
}

impl From<(&ColorPalette, &SymbolPalette)> for ConfigV1 {
    fn from(palettes: (&ColorPalette, &SymbolPalette)) -> Self {
        ConfigV1 {
            color_palette: palettes.0.clone(),
            symbol_palette: palettes.1.clone(),
            char_map: CharMap::default(),
        }
    }
}
