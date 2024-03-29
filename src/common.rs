mod action;
mod clipboard;
mod config;
mod help;
mod input;
mod mode;
mod program;
mod scene;
mod selected_info;
mod sprite;
mod texel;

pub mod fio; // file io

pub use action::{Action, Layout, MetadataType, LAYOUT_WORDS, METADATA_TYPES};
pub use clipboard::{Clipboard, ClipboardOp};
pub use config::{Config, ConfigV2};
pub use help::*;
pub use input::*;
pub use mode::{Mode, OnQuit, SelectMode};
pub use program::run;
pub use scene::{Scene, SceneExt};
pub use selected_info::{SelectedInfo, SELECTED_INFO_TEMPLATE};
pub use sprite::SpriteExt;
pub use texel::TexelExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidCommand,
    InvalidParam(&'static str),
    Execution(String),
}

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(err: T) -> Self {
        Error::Execution(err.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCommand => write!(f, "Error: Invalid command"),
            Error::InvalidParam(p) => write!(f, "Error: {}", p),
            Error::Execution(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error {
    pub fn execution(src: &'static str) -> Self {
        Error::Execution(String::from(src))
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

pub fn index_from_one(index: usize) -> i32 {
    match index {
        0..=8 => (index + 1) as i32,
        9 => 0,
        _ => index as i32,
    }
}

pub fn shortened_str(source: &str, max_chars: usize) -> (&str, bool) {
    if source.len() > max_chars {
        (&source[0..max_chars - 3], true)
    } else {
        (source, false)
    }
}
