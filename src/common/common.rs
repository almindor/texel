use crate::components::{Sprite, Translation};
use crate::resources::Mode;
use serde::{Deserialize, Serialize};
use specs::{Join, ReadStorage};
use std::env::current_dir;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Texel {
    pub x: i32,
    pub y: i32,
    pub symbol: char,
    pub color: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct TexelDiff {
    pub index: usize,
    pub texel: Texel,
}

#[derive(Debug)]
pub enum Action {
    None,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    SelectNext(bool), // select next keeping old if true
    Import(Sprite),
    Load(String),
    Save(String),
    Translate(Translation),
    Delete,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

impl From<&str> for Action {
    fn from(source: &str) -> Self {
        match source {
            "import" => Action::Import(Sprite::default()),
            "load" => Action::Load(String::default()),
            "save" => Action::Save(String::default()),
            "translate" => Action::Translate(Translation::default()),
            "delete" => Action::Delete,
            "deselect" => Action::Deselect,
            "quit" | "q" => Action::SetMode(Mode::Quitting(false)),
            "quit!" | "q!" => Action::SetMode(Mode::Quitting(true)),
            _ => Action::None,
        }
    }
}

impl From<Option<&str>> for Action {
    fn from(source: Option<&str>) -> Self {
        match source {
            Some(s) => Action::from(s),
            None => Action::None,
        }
    }
}

impl Action {
    pub fn is_some(&self) -> bool {
        match self {
            Action::None => false,
            _ => true,
        }
    }

    pub fn complete_word(part: &str) -> Option<&'static str> {
        const ACTION_WORDS: [&'static str; 8] = [
            "import",
            "load",
            "save",
            "translate",
            "delete",
            "deselect",
            "quit",
            "quit!",
        ];

        for word in &ACTION_WORDS {
            if word.starts_with(part) {
                return Some(word);
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidCommand,
    InvalidParam(&'static str),
    ExecutionError(String),
}

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(err: T) -> Self {
        Error::ExecutionError(err.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCommand => write!(f, "Error: Invalid command"),
            Error::InvalidParam(p) => write!(f, "Error: {}", p),
            Error::ExecutionError(e) => write!(f, "Error: {}", e),
        }
    }
}

// TODO: figure out a 0-copy way to keep scene serializable/deserializable
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scene {
    sprites: Vec<Sprite>,
}

impl<'a> From<&ReadStorage<'a, Sprite>> for Scene {
    fn from(storage: &ReadStorage<Sprite>) -> Self {
        let mut sprites = Vec::new();

        for sprite in storage.join() {
            sprites.push(sprite.clone());
        }

        Scene { sprites }
    }
}

pub const fn goto(x: i32, y: i32) -> termion::cursor::Goto {
    // TODO: figure out best way to handle this
    let u_x = x as u16;
    let u_y = y as u16;

    termion::cursor::Goto(u_x, u_y)
}

pub fn cwd_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cwd = current_dir()?;
        Ok(cwd.join(path))
    }
}

pub fn complete_filename(raw_path: &str) -> Result<Option<String>, Error> {
    let loc_path = Path::new(raw_path);
    let abs_path = cwd_path(loc_path)?;
    let loc_parent = loc_path.parent().unwrap_or(Path::new(""));
    let abs_parent = abs_path.parent().unwrap_or(Path::new("/"));

    if let Some(name) = loc_path.file_name() {
        let str_name = name.to_str().unwrap_or("");
        for entry in read_dir(abs_parent)? {
            let full_path = entry?.path();
            if let Some(os_fn) = full_path.file_name() {
                let fn_path = os_fn.to_str().ok_or(Error::ExecutionError(String::from(
                    "Invalid utf-8 path string",
                )))?;
                if fn_path.starts_with(str_name) {
                    let joined = loc_parent.join(os_fn);
                    let fn_str = joined.to_str().ok_or(Error::ExecutionError(String::from(
                        "Invalid utf-8 path string",
                    )))?;
                    let fn_string = String::from(fn_str);
                    match full_path.is_dir() {
                        true => return Ok(Some(fn_string + "/")),
                        false => return Ok(Some(fn_string)),
                    }
                }
            }
        }
    }

    Ok(None)
}
