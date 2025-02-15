use crate::common::fio::ExportFormat;
use crate::common::{ClipboardOp, Error, Mode, OnQuit};
use std::collections::HashMap;
use texel_types::{ColorMode, Position2D, SymbolStyle, Translation, Which};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    None,
    Column(usize, (u16, u16)), // number of columns, padding size x, y
    Random,
}

impl From<&str> for Layout {
    fn from(source: &str) -> Self {
        match source {
            "column" => Layout::Column(0, (0, 0)),
            "random" => Layout::Random,
            _ => Layout::None,
        }
    }
}

pub const LAYOUT_WORDS: [&str; 2] = ["column", "random"];

#[derive(Debug)]
pub enum MetadataType {
    Id(Option<u32>),
    Labels(HashMap<String, String>),
}

impl MetadataType {
    pub fn is_id(&self) -> bool {
        matches!(self, Self::Id(_))
    }

    pub fn parse_id(source: &str) -> Result<Self, Error> {
        match source {
            "none" => Ok(MetadataType::Id(None)),
            _ => Ok(MetadataType::Id(Some(source.parse()?))),
        }
    }

    pub fn parse_labels(source: &str) -> Result<Self, Error> {
        let mut labels = HashMap::new();

        for label in source.split(',').map(|s| s.trim()) {
            let mut parts = label.split('=').map(|s| s.trim());
            let key = parts.next().ok_or_else(|| Error::execution("Invalid label key"))?;
            let value = parts.next().ok_or_else(|| Error::execution("Invalid label value"))?;

            labels.insert(key.into(), value.into());
        }

        Ok(MetadataType::Labels(labels))
    }
}

pub const METADATA_TYPES: [&str; 2] = ["id", "labels"];

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
    New(bool),
    NewObject,
    Duplicate(usize),
    Clipboard(ClipboardOp),
    ToggleMetadata,
    SetMetadata(MetadataType),
    Cancel,
    ClearError,
    SetMode(Mode),
    ReverseMode,
    Deselect,
    PickColor(ColorMode),
    SwapColor,
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
    WriteAndQuit(Option<String>),
    Export(ExportFormat, String),
    Translate(Translation),
    Layout(Layout),
    Delete,
    Undo,
    Redo,
    ShowHelp(usize),
    Bookmark(usize, bool), // index and "set"
    Tutorial,
    ClearBlank, // clears "blank" texels from sprite/selection
}

impl From<&str> for Action {
    fn from(source: &str) -> Self {
        match source {
            "new" | "n" => Action::New(false),
            "new!" | "n!" => Action::New(true),
            "read" | "r" => Action::Read(String::default()),
            "write" | "w" => Action::Write(None),
            "translate" => Action::Translate(Translation::default()),
            "delete" => Action::Delete,
            "deselect" => Action::Deselect,
            "quit" | "q" => Action::SetMode(Mode::Quitting(OnQuit::Check)),
            "quit!" | "q!" => Action::SetMode(Mode::Quitting(OnQuit::Force)),
            "x" => Action::WriteAndQuit(None),
            "help" | "h" => Action::ShowHelp(0),
            "export" => Action::Export(ExportFormat::default(), String::default()),
            "tutorial" => Action::Tutorial,
            "clear_blank" => Action::ClearBlank,
            "duplicate" => Action::Duplicate(1),
            "layout" => Action::Layout(Layout::None),
            "set" => Action::SetMetadata(MetadataType::Id(None)),
            "metadata" => Action::ToggleMetadata,
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
        !matches!(self, Self::None)
    }

    pub fn complete_word(part: &str) -> Option<&'static str> {
        const ACTION_WORDS: [&str; 16] = [
            "new",
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
            "set",
            "metadata",
        ];

        return ACTION_WORDS.iter().find(|&word| word.starts_with(part)).copied();
    }
}
