use crate::components::{Position, Sprite, Translation};
use crate::resources::Mode;
use serde::{Deserialize, Serialize};
use specs::{Join, ReadStorage, WriteStorage};
use std::env::current_dir;
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
    pub objects: Vec<(Sprite, Position)>,
}

impl<'a> From<(&ReadStorage<'a, Sprite>, &WriteStorage<'a, Position>)> for Scene {
    fn from(storage: (&ReadStorage<'a, Sprite>, &WriteStorage<'a, Position>)) -> Self {
        let mut objects = Vec::new();

        for (sprite, pos) in storage.join() {
            objects.push((sprite.clone(), pos.clone()));
        }

        Scene { objects }
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

pub fn path_base(path: &str) -> String {
    if let Some(base) = Path::new(path).parent() {
        return String::from(base.to_str().unwrap_or(""));
    }

    String::default()
}
