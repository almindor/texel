use crate::components::Translation;
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
    SetMode(Mode),
    ReverseMode,
    Translate(Translation),
    Delete,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter, AsRefStr)]
pub enum Command {
    None,
    Engage,
    Clear,
    Quit,
    Cancel,
    Perform(Action),
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
