use crate::common::{Config, Error, Scene};
use crate::components::Sprite;
use crate::exporters::{Exporter, Plaintext};
use libflate::gzip::{Decoder, Encoder};
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Loaded {
    Sprite(Sprite),
    Scene(Scene),
    // config is not needed to be loaded "generically"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Txt,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Txt
    }
}

// pub const EXPORT_FORMAT_LIST: [&str; 1] = [
//     "txt",
// ];

pub fn export_to_file(scene: Scene, format: ExportFormat, path: &str) -> Result<(), Error> {
    let abs_path = to_abs_path_with_ext(path, "txt")?;
    let mut file = File::create(abs_path)?;

    match format {
        ExportFormat::Txt => Plaintext::export(scene, &mut file)?,
    }

    Ok(())
}

pub fn scene_to_file(scene: &Scene, path: &str) -> Result<(), Error> {
    let abs_path = to_abs_path_with_ext(path, "rgz")?;
    let file = std::fs::File::create(abs_path)?;
    let mut encoder = Encoder::new(file)?;
    let ronified = ron::ser::to_string(&scene)?;

    use std::io::Write;
    encoder.write_all(ronified.as_ref())?;
    encoder.finish().into_result()?;

    Ok(())
}

pub fn load_from_file(path: &str) -> Result<Loaded, Error> {
    let path = Path::new(path);

    match path.extension() {
        Some(ext) => match ext
            .to_str()
            .ok_or_else(|| Error::execution("Unable to parse extension"))?
        {
            "rgz" => scene_from_rgz_file(path),
            _ => Ok(Loaded::Sprite(sprite_from_txt_file(path)?)),
        },
        None => Ok(Loaded::Sprite(sprite_from_txt_file(path)?)),
    }
}

pub fn from_config_file(path: &Path) -> Result<Config, Error> {
    let abs_path = cwd_path(path)?;
    let file = std::fs::File::open(abs_path)?;

    Ok(ron::de::from_reader(file)?)
}

fn sprite_from_txt_file(path: &Path) -> Result<Sprite, Error> {
    let abs_path = cwd_path(path)?;

    let ext = abs_path.extension().unwrap_or_default();

    if ext == "txt" {
        Ok(Sprite::from_txt_file(&abs_path)?)
    } else {
        Err(Error::execution("Unknown file type"))
    }
}

fn scene_from_rgz_file(abs_path: &Path) -> Result<Loaded, Error> {
    let file = std::fs::File::open(abs_path)?;

    scene_from_rgz_stream(file)
}

pub fn scene_from_rgz_stream(stream: impl std::io::Read) -> Result<Loaded, Error> {
    let decoder = Decoder::new(stream)?;
    let scene: Scene = ron::de::from_reader(decoder)?;

    Ok(Loaded::Scene(scene))
}

pub fn cwd_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(path))
    }
}

pub fn path_base(path: &str) -> String {
    if let Some(base) = Path::new(path).parent() {
        return String::from(base.to_str().unwrap_or(""));
    }

    String::default()
}

fn to_abs_path_with_ext(path: &str, ext: &str) -> Result<PathBuf, std::io::Error> {
    let raw_path = if Path::new(&path).extension() != Some(std::ffi::OsStr::new(ext)) {
        Path::new(&path).with_extension(ext)
    } else {
        PathBuf::from(path)
    };

    cwd_path(&raw_path)
}
