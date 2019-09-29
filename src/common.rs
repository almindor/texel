use crate::components::{Position, Selection, Sprite};
use serde::{Deserialize, Serialize};
use specs::{Entities, Join, ReadStorage, WriteStorage};
use std::env::current_dir;
use std::path::{Path, PathBuf};

mod action;

pub use action::Action;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Texel {
    pub x: i32,
    pub y: i32,
    pub symbol: char,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct TexelDiff {
    pub index: usize,
    pub texel: Texel,
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
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Scene {
    pub objects: Vec<(Sprite, Position, bool)>,
}

impl<'a>
    From<(
        &Entities<'a>,
        &ReadStorage<'a, Sprite>,
        &WriteStorage<'a, Position>,
        &ReadStorage<'a, Selection>,
    )> for Scene
{
    fn from(
        storage: (
            &Entities,
            &ReadStorage<'a, Sprite>,
            &WriteStorage<'a, Position>,
            &ReadStorage<'a, Selection>,
        ),
    ) -> Self {
        let mut objects = Vec::new();
        let (e, sp, p, s) = storage;

        for (entity, sprite, pos) in (e, sp, p).join() {
            objects.push((sprite.clone(), pos.clone(), s.contains(entity)));
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
