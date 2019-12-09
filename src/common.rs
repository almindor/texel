use std::env::current_dir;
use std::path::{Path, PathBuf};

mod action;
mod clipboard;
mod config;
mod help;
mod input;
mod mode;
mod program;
mod scene;
mod texel;

pub mod fio; // file io

pub use action::Action;
pub use clipboard::{Clipboard, ClipboardOp};
pub use config::{Config, ConfigV1};
pub use help::*;
pub use input::{CharMap, Event, InputEvent, InputMap};
pub use mode::Mode;
pub use program::run;
pub use scene::{scene_from_objects, Scene};
pub use texel::*;

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

impl Error {
    pub fn execution(src: &'static str) -> Self {
        Error::ExecutionError(String::from(src))
    }
}

pub fn add_max(u: usize, i: i32, m: usize) -> Option<usize> {
    let result = if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    };

    if let Some(val) = result {
        if val > m {
            return None;
        }
    }

    result
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

pub fn index_from_one(index: usize) -> i32 {
    if index < 9 {
        (index + 1) as i32
    } else if index == 9 {
        0
    } else {
        index as i32
    }
}
