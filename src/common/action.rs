use crate::common::SymbolStyle;
use crate::components::Translation;
use crate::resources::{ColorMode, Mode};

#[derive(Debug)]
pub enum Action {
    None,
    NewObject,
    Cancel,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    ApplyColor(ColorMode),
    ApplySymbol(char),
    ApplyStyle(SymbolStyle),
    SelectNext(bool), // select next keeping old if true
    Read(String),
    Write(Option<String>),
    Translate(Translation),
    Delete,
    Undo,
    Redo,
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
            "quit" | "q" => Action::SetMode(Mode::Quitting(false)),
            "quit!" | "q!" => Action::SetMode(Mode::Quitting(true)),
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
            | Action::SetMode(_)
            | Action::Write(_) => false,
            _ => true,
        }
    }

    pub fn complete_word(part: &str) -> Option<&'static str> {
        const ACTION_WORDS: [&str; 8] = [
            "import",
            "read",
            "write",
            "translate",
            "delete",
            "deselect",
            "quit",
            "quit!",
        ];

        for word in &ACTION_WORDS {
            if word.starts_with(part) {
                return Some(word);
            }
        }

        None
    }
}
