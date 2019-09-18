use crate::common::Error;
use crate::components::Sprite;
use std::path::Path;

pub struct Loader;

impl Loader {
    pub fn from_files(paths: &[String]) -> Result<Vec<Sprite>, Error> {
        let mut result = Vec::new();

        for path in paths {
            result.push(Self::from_file(path)?);
        }

        Ok(result)
    }

    pub fn from_file(path: &str) -> Result<Sprite, Error> {
        if path.ends_with("txt") {
            Ok(Sprite::from_file(Path::new(path))?)
        } else {
            Err(Error::ExecutionError(String::from("Unknown file type")))
        }
    }
}
