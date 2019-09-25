use crate::components::{Sprite, Translation};
use crate::resources::Mode;
// use undo::Command;

#[derive(Debug)]
pub enum Action {
    None,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    SelectNext(bool), // select next keeping old if true
    Import(Sprite),
    Load(String),
    Save(String),
    Translate(Translation),
    Delete,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

impl From<&str> for Action {
    fn from(source: &str) -> Self {
        match source {
            "import" => Action::Import(Sprite::default()),
            "load" => Action::Load(String::default()),
            "save" => Action::Save(String::default()),
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

// impl<'a> Command<ActionSystemData<'a>> for Action {
//     fn apply(&mut self, s: &mut ActionSystemData) -> undo::Result {
//         Ok(())
//     }

//     fn undo(&mut self, s: &mut ActionSystemData) -> undo::Result {
//         Ok(())
//     }
// }

impl Action {
    pub fn is_some(&self) -> bool {
        match self {
            Action::None => false,
            _ => true,
        }
    }

    pub fn complete_word(part: &str) -> Option<&'static str> {
        const ACTION_WORDS: [&'static str; 8] = [
            "import",
            "load",
            "save",
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
