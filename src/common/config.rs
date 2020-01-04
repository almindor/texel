use crate::common::{CharMap, Error};
use crate::resources::{ColorPalette, SymbolPalette};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            Self::V1(config) => check_new_keys(config),
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

impl From<ConfigV1> for Config {
    fn from(v1: ConfigV1) -> Config {
        Config::V1(v1)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigV1 {
    pub color_palette: ColorPalette,
    pub symbol_palette: SymbolPalette,
    pub char_map: CharMap,
}

// goes over defaults and adds them if they're not existing yet
// this is a "minor version" upgrade process
fn check_new_keys(mut config: ConfigV1) -> ConfigV1 {
    let defaults = CharMap::default();

    for (key, value) in &defaults.0 {
        if config.char_map.0.values().find(|v| *v == value).is_none() && !config.char_map.0.contains_key(key) {
            config.char_map.0.insert(*key, *value);
        }
    }

    config
}
