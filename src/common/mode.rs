use serde::{Deserialize, Serialize};
use texel_types::ColorMode;

// describes "how" to quit (normal/check, force, save & quit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnQuit {
    Check,
    Force,
    Save,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectMode {
    Object,
    Region,
}

impl Default for SelectMode {
    fn default() -> Self {
        Self::Object
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Object(SelectMode),
    Color(ColorMode),
    SelectColor(usize, ColorMode), // index for which color 0 -> 16 (0x0 to 0xF)
    SelectSymbol(usize),           // index for which symbol 0 -> 16 (0x0 to 0xF)
    Edit,
    Write,
    Command,
    Quitting(OnQuit), // true for force quit
    Help(usize),      // help index
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object(SelectMode::default())
    }
}

impl Mode {
    // basic bit-mapping for each mode except quitting
    pub fn to_bits(&self) -> u8 {
        match self {
            Mode::Object(_) => 0b1000_0000,
            Mode::Color(_) => 0b0100_0000,
            Mode::SelectColor(_, _) => 0b0010_0000,
            Mode::SelectSymbol(_) => 0b0001_0000,
            Mode::Edit => 0b0000_1000,
            Mode::Write => 0b0000_0100,
            Mode::Command => 0b0000_0010,
            Mode::Help(_) => 0b0000_0001,
            _ => 0u8,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Mode::Object(SelectMode::Object) => "OBJECT[OBJECT]",
            Mode::Object(SelectMode::Region) => "OBJECT[REGION]",
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
