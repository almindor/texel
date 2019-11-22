use crate::common::{cwd_path, Config, Error, Scene};
use crate::components::Sprite;
use libflate::gzip::Decoder;
use std::path::Path;

pub struct Loader;

#[derive(Debug)]
pub enum Loaded {
    Sprite(Sprite),
    Scene(Scene),
    // config is not needed to be loaded "generically"
}

impl Loader {
    pub fn from_file(path: &str) -> Result<Loaded, Error> {
        match Path::new(path).extension() {
            Some(ext) => match ext
                .to_str()
                .ok_or_else(|| Error::execution("Unable to parse extension"))?
            {
                "rgz" => Self::from_rgz_file(path),
                _ => Ok(Loaded::Sprite(Self::from_txt_file(path)?)),
            },
            None => Ok(Loaded::Sprite(Self::from_txt_file(path)?)),
        }
    }

    pub fn from_config_file(path: &Path) -> Result<Config, Error> {
        let abs_path = cwd_path(path)?;
        let file = std::fs::File::open(abs_path)?;

        Ok(ron::de::from_reader(file)?)
    }

    fn from_rgz_file(path: &str) -> Result<Loaded, Error> {
        let abs_path = cwd_path(Path::new(path))?;
        let file = std::fs::File::open(abs_path)?;

        let decoder = Decoder::new(file)?;
        let scene: Scene = ron::de::from_reader(decoder)?;

        Ok(Loaded::Scene(scene))
    }

    pub fn from_txt_file(path: &str) -> Result<Sprite, Error> {
        if path.ends_with("txt") {
            Ok(Sprite::from_file(Path::new(path))?)
        } else {
            Err(Error::execution("Unknown file type"))
        }
    }
}
