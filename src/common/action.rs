use crate::common::fio::ExportFormat;
use crate::common::{ClipboardOp, Mode, OnQuit};
use texel_types::{ColorMode, Position2D, SymbolStyle, Translation, Which};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    None,
    Column(usize, i32), // number of columns, padding size
    Random,
}

impl From<&str> for Layout {
    fn from(source: &str) -> Self {
        match source {
            "column" => Layout::Column(0, 0),
            "random" => Layout::Random,
            _ => Layout::None,
        }
    }
}

pub const LAYOUT_WORDS: [&str; 2] = [
    "column",
    "random",
];

#[derive(Debug)]
pub enum Action {
    None,
    NewObject,
    Duplicate(usize),
    Clipboard(ClipboardOp),
    Cancel,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    ApplyColor(ColorMode),
    ApplySymbol(char),
    ApplyStyle(SymbolStyle),
    ApplyRegion,               // takes "selected region" and translates into selection on objects
    SelectFrame(Which<usize>), // next/prev + index into number
    DeleteFrame,
    NewFrame,
    SelectObject(Which<Position2D>, bool), // select next keeping old if true
    SelectRegion,
    Read(String),
    Write(Option<String>),
    Export(ExportFormat, String),
    Translate(Translation),
    Layout(Layout),
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
            "duplicate" => Action::Duplicate(1),
            "layout" => Action::Layout(Layout::None),
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
        const ACTION_WORDS: [&str; 13] = [
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
            "duplicate",
            "layout",
        ];

        for word in &ACTION_WORDS {
            if word.starts_with(part) {
                return Some(word);
            }
        }

        None
    }
}
