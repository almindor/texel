use crate::common::{CharMap, Error, ModesCharMap};
use crate::resources::{ColorPalette, SymbolPalette};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Config {
    V1(ConfigV1),
    V2(ConfigV2),
}

impl Default for Config {
    fn default() -> Self {
        Config::V2(ConfigV2::default())
    }
}

impl Config {
    pub fn current(self) -> ConfigV2 {
        match self {
            Self::V1(config) => upgrade_v1_to_v2(config),
            Self::V2(config) => config,
        }
    }

    pub fn to_config_file(&self, path: &Path) -> Result<(), Error> {
        let parent = path
            .parent()
            .ok_or_else(|| Error::execution("Unable to create config diff"))?;
        std::fs::create_dir_all(parent)?;

        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, serialized)?;

        Ok(())
    }
}

impl From<ConfigV2> for Config {
    fn from(v2: ConfigV2) -> Config {
        Config::V2(v2)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigV1 {
    pub color_palette: ColorPalette,
    pub symbol_palette: SymbolPalette,
    pub char_map: CharMap,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigV2 {
    pub color_palette: ColorPalette,
    pub symbol_palette: SymbolPalette,
    pub char_map: ModesCharMap,
}

fn upgrade_v1_to_v2(v1: ConfigV1) -> ConfigV2 {
    ConfigV2 {
        color_palette: v1.color_palette,
        symbol_palette: v1.symbol_palette,
        char_map: ModesCharMap::from(v1.char_map),
    }
}
