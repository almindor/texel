use crate::components::{Sprite, Translation};
use crate::resources::Mode;
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
    Import(Sprite),
    Translate(Translation),
    Delete,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
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
