use serde::{Deserialize, Serialize};
use texel_types::ColorMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Object,
    Color(ColorMode),
    SelectColor(usize, ColorMode), // index for which color 0 -> 16 (0x0 to 0xF)
    SelectSymbol(usize),           // index for which symbol 0 -> 16 (0x0 to 0xF)
    Edit,
    Write,
    Command,
    Quitting(bool), // true for force quit
    Help(usize), // help index
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object
    }
}

impl Mode {
    pub fn to_str(&self) -> &'static str {
        match self {
            Mode::Object => "OBJECT",
            Mode::Color(ColorMode::Fg) => "COLOR[FG]",
            Mode::Color(ColorMode::Bg) => "COLOR[BG]",
            Mode::SelectColor(_, ColorMode::Fg) => "COLOR[SET-FG]", // TODO: construct static numbered index
            Mode::SelectColor(_, ColorMode::Bg) => "COLOR[SET-BG]", // TODO: construct static numbered index
            Mode::SelectSymbol(_) => "SYMBOL[SET]",                 // TODO: construct static numbered index
            Mode::Edit => "EDIT",
            Mode::Write => "WRITE",
            Mode::Command => "COMMAND",
            Mode::Quitting(_) => "QUITTING",
            Mode::Help(_) => "HELP",
        }
    }
}
