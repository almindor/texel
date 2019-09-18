use crate::components::Translation;
use crate::resources::Mode;
use std::path::PathBuf;
use strum_macros::{AsRefStr, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Texel {
    pub x: i32,
    pub y: i32,
    pub symbol: char,
    pub color: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TexelDiff {
    pub index: usize,
    pub texel: Texel,
}

#[derive(Debug, PartialEq, Eq, EnumIter, AsRefStr)]
pub enum Action {
    None,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    SelectNext(bool), // select next keeping old if true
    Import(PathBuf),
    Translate(Translation),
    Delete,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    InvalidCommand,
    InvalidParam(&'static str),
    ExecutionError(String),
}

impl<T> From<T> for ExecuteError
where
    T: std::error::Error,
{
    fn from(err: T) -> Self {
        ExecuteError::ExecutionError(err.to_string())
    }
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::InvalidCommand => write!(f, "Error: Invalid command"),
            ExecuteError::InvalidParam(p) => write!(f, "Error: {}", p),
            ExecuteError::ExecutionError(e) => write!(f, "Error: {}", e),
        }
    }
}

pub const fn goto(x: i32, y: i32) -> termion::cursor::Goto {
    // TODO: figure out best way to handle this
    let u_x = x as u16;
    let u_y = y as u16;

    termion::cursor::Goto(u_x, u_y)
}

pub fn make_ascii_titlecase(s: &mut str) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

pub fn to_ascii_titlecase(s: &str) -> String {
    let mut result = String::from(s);
    make_ascii_titlecase(&mut result);

    result
}
