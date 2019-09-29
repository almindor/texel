use crate::common::{Error, Scene, cwd_path};
use crate::components::{Sprite};
use libflate::gzip::{Decoder};
use std::path::Path;

pub struct Loader;

#[derive(Debug)]
pub enum Loaded {
    Sprite(Sprite),
    Scene(Scene),
}

impl Loader {
    pub fn from_file(path: &str) -> Result<Loaded, Error> {
        match Path::new(path).extension() {
            Some(ext) => if ext == "rgz" {
                Self::from_rgz_file(path)
            } else {
                Ok(Loaded::Sprite(Self::from_txt_file(path)?))
            }
            None => Ok(Loaded::Sprite(Self::from_txt_file(path)?))
        }
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
            Err(Error::ExecutionError(String::from("Unknown file type")))
        }
    }
}
