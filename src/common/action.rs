use crate::common::fio::ExportFormat;
use crate::common::{ClipboardOp, Mode, OnQuit};
use texel_types::{ColorMode, Position2D, SymbolStyle, Translation, Which};

#[derive(Debug)]
pub enum Action {
    None,
    NewObject,
    Clipboard(ClipboardOp),
    Cancel,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    ApplyColor(ColorMode),
    ApplySymbol(char),
    ApplyStyle(SymbolStyle),
    SelectFrame(Which<usize>), // next/prev + index into number
    DeleteFrame,
    NewFrame,
    SelectObject(Which<Position2D>, bool), // select next keeping old if true
    Read(String),
    Write(Option<String>),
    Export(ExportFormat, String),
    Translate(Translation),
    Delete,
    Undo,
    Redo,
    ShowHelp(usize),
    Tutorial,
    ClearBlank, // clears "blank" texels from sprite/selection
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

impl From<&str> for Action {
    fn from(source: &str) -> Self {
        match source {
            "read" | "r" => Action::Read(String::default()),
            "write" | "w" => Action::Write(None),
            "translate" => Action::Translate(Translation::default()),
            "delete" => Action::Delete,
            "deselect" => Action::Deselect,
            "quit" | "q" => Action::SetMode(Mode::Quitting(OnQuit::Check)),
            "quit!" | "q!" => Action::SetMode(Mode::Quitting(OnQuit::Force)),
            "x" => Action::SetMode(Mode::Quitting(OnQuit::Save)),
            "help" | "h" => Action::ShowHelp(0),
            "export" => Action::Export(ExportFormat::default(), String::default()),
            "tutorial" => Action::Tutorial,
            "clear_blank" => Action::ClearBlank,
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

    pub fn is_reverse_mode(&self) -> bool {
        match self {
            Action::ReverseMode => true,
            _ => false,
        }
    }

    pub fn keeps_history(&self) -> bool {
        match self {
            Action::None
            | Action::Undo
            | Action::Redo
            | Action::ClearError
            | Action::ReverseMode
            | Action::ShowHelp(_)
            | Action::SetMode(_)
            | Action::Write(_) => false,
            _ => true,
        }
    }

    pub fn complete_word(part: &str) -> Option<&'static str> {
        const ACTION_WORDS: [&str; 11] = [
            "read",
            "write",
            "translate",
            "delete",
            "deselect",
            "quit",
            "quit!",
            "help",
            "export",
            "tutorial",
            "clear_blank",
        ];

        for word in &ACTION_WORDS {
            if word.starts_with(part) {
                return Some(word);
            }
        }

        None
    }
}
