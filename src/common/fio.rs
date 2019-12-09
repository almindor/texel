use crate::common::{cwd_path, Config, Error, Scene};
use crate::components::Sprite;
use libflate::gzip::{Decoder, Encoder};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Loaded {
    Sprite(Sprite),
    Scene(Scene),
    // config is not needed to be loaded "generically"
}

pub fn to_file(scene: &Scene, path: &str) -> Result<(), Error> {
    let ronified = ron::ser::to_string(&scene)?;
    let raw_path = if Path::new(&path).extension() != Some(std::ffi::OsStr::new("rgz")) {
        Path::new(&path).with_extension("rgz")
    } else {
        PathBuf::from(path)
    };
    let abs_path = cwd_path(&raw_path)?;
    let file = std::fs::File::create(abs_path)?;
    let mut encoder = Encoder::new(file)?;

    use std::io::Write;
    encoder.write_all(ronified.as_ref())?;
    encoder.finish().into_result()?;

    Ok(())
}

pub fn scene_from_file(path: &str) -> Result<Loaded, Error> {
    match Path::new(path).extension() {
        Some(ext) => match ext
            .to_str()
            .ok_or_else(|| Error::execution("Unable to parse extension"))?
        {
            "rgz" => scene_from_rgz_file(path),
            _ => Ok(Loaded::Sprite(from_txt_file(path)?)),
        },
        None => Ok(Loaded::Sprite(from_txt_file(path)?)),
    }
}

pub fn from_config_file(path: &Path) -> Result<Config, Error> {
    let abs_path = cwd_path(path)?;
    let file = std::fs::File::open(abs_path)?;

    Ok(ron::de::from_reader(file)?)
}

pub fn from_txt_file(path: &str) -> Result<Sprite, Error> {
    if path.ends_with("txt") {
        let abs_path = cwd_path(Path::new(path))?;
        Ok(Sprite::from_txt_file(&abs_path)?)
    } else {
        Err(Error::execution("Unknown file type"))
    }
}

fn scene_from_rgz_file(path: &str) -> Result<Loaded, Error> {
    let abs_path = cwd_path(Path::new(path))?;
    let file = std::fs::File::open(abs_path)?;

    scene_from_rgz_stream(file)
}

pub fn scene_from_rgz_stream(stream: impl std::io::Read) -> Result<Loaded, Error> {
    let decoder = Decoder::new(stream)?;
    let scene: Scene = ron::de::from_reader(decoder)?;

    Ok(Loaded::Scene(scene))
}