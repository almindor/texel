use crate::common::{fio, topic_index, Action, Error, Event, InputEvent, Layout, MetadataType};
use crate::components::Translation;
use std::iter::Peekable;
use std::str::SplitAsciiWhitespace;

mod auto_complete;

use auto_complete::{AutoComplete, Completion};

const DEFAULT_CMD_CAPACITY: usize = 4096; // coz I said so!
const MAX_HISTORY_ENTRIES: usize = 255; // coz I said so too!

#[derive(Debug)]
pub struct CmdLine {
    cmd: String,
    cursor_pos: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    auto_complete: AutoComplete,
}

impl Default for CmdLine {
    fn default() -> Self {
        CmdLine {
            cmd: String::with_capacity(DEFAULT_CMD_CAPACITY),
            cursor_pos: 0,
            history: Vec::with_capacity(MAX_HISTORY_ENTRIES),
            history_index: None,
            auto_complete: AutoComplete::new(),
        }
    }
}

impl CmdLine {
    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn input(&mut self, event: InputEvent) -> Result<Action, Error> {
        let mut clear_ac = true;
        let result = match event.0 {
            Event::Cancel => Ok(Action::ReverseMode),
            Event::Backspace => self.remove(),

            Event::ArrowUp => self.previous(),
            Event::ArrowDown => self.next(),
            Event::ArrowLeft => self.move_cursor(-1),
            Event::ArrowRight => self.move_cursor(1),

            Event::Confirm => self.parse(),
            Event::SelectObject(_, false) => {
                clear_ac = false;
                self.auto_complete()
            }

            // otherwise get char and handle
            _ => {
                if let Some(c) = event.1 {
                    self.append(c)
                } else {
                    Ok(Action::None)
                }
            }
        };

        if clear_ac {
            self.auto_complete.clear();
        }

        // flush on anything but Command::None
        if let Ok(Action::None) = &result {
        } else {
            self.flush();
        }

        result
    }

    fn flush(&mut self) {
        self.history.push(self.cmd.clone());
        self.history_index = Some(self.history.len() - 1);
        self.cmd.clear();
        self.cursor_pos = 0;
    }

    fn move_cursor(&mut self, diff: i32) -> Result<Action, Error> {
        if let Some(new_pos) = crate::common::add_max(self.cursor_pos, diff, self.cmd.len()) {
            self.cursor_pos = new_pos;
        } // otherwise ignore

        Ok(Action::None)
    }

    fn previous(&mut self) -> Result<Action, Error> {
        if !self.history.is_empty() {
            if self.history_index.is_none() {
                self.history_index = Some(self.history.len() - 1);
            }

            if let Some(index) = self.history_index {
                self.cmd.clone_from(&self.history[index]);
                if index > 0 {
                    self.history_index = Some(index - 1);
                }
                self.cursor_pos = self.cmd.len();
            }
        }

        Ok(Action::None)
    }

    fn next(&mut self) -> Result<Action, Error> {
        if let Some(index) = self.history_index {
            if index < self.history.len() - 1 {
                self.cmd.clone_from(&self.history[index + 1]);
                self.history_index = Some(index + 1);
            } else {
                self.cmd.clear();
                self.history_index = None;
            }

            self.cursor_pos = self.cmd.len();
        }

        Ok(Action::None)
    }

    fn append(&mut self, c: char) -> Result<Action, Error> {
        if self.cursor_pos != self.cmd.len() {
            self.cmd.insert(self.cursor_pos, c);
        } else {
            self.cmd.push(c);
        }
        self.cursor_pos += 1;

        Ok(Action::None)
    }

    fn remove(&mut self) -> Result<Action, Error> {
        if self.cursor_pos == self.cmd.len() {
            self.cmd.pop();
        } else if self.cursor_pos > 0 {
            self.cmd.remove(self.cursor_pos - 1);
        }

        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }

        Ok(Action::None)
    }

    fn auto_complete(&mut self) -> Result<Action, Error> {
        let parts: Vec<&str> = self.cmd.split_ascii_whitespace().collect();

        if parts.len() == 1 {
            if let Some(word) = parts.first() {
                if let Some(cmd) = Action::complete_word(word) {
                    self.cmd = fio::path_base(&self.cmd) + cmd;
                    self.cursor_pos = self.cmd.len();
                }
            }
        } else if let Some(cmd) = parts.first() {
            // if we have something here, and count != 1 parts.count() must be >= 1
            if let Some(completion) = match *cmd {
                "export" | "read" | "write" | "w" | "r" => self
                    .auto_complete
                    .complete_filename(parts.last().unwrap_or_else(|| &"."))?,
                "set" => self
                    .auto_complete
                    .complete_from_list(parts.last().unwrap_or_else(|| &""), &crate::common::METADATA_TYPES),
                "help" => self
                    .auto_complete
                    .complete_from_list(parts.last().unwrap_or_else(|| &""), &crate::common::HELP_TOPICS),
                "layout" => self
                    .auto_complete
                    .complete_from_list(parts.last().unwrap_or_else(|| &""), &crate::common::LAYOUT_WORDS),
                _ => None,
            } {
                match completion {
                    Completion::Filename(word) | Completion::Parameter(word) => {
                        self.cmd = String::from(*cmd) + " " + &word;
                    }
                    Completion::Directory(dir) => {
                        self.cmd = String::from(*cmd) + " " + &dir + "/";
                    }
                }

                self.cursor_pos = self.cmd.len();
            }
        }

        Ok(Action::None)
    }

    fn parse(&mut self) -> Result<Action, Error> {
        self.cursor_pos = 0; // reset even in case of errors
        let mut parts = self.cmd.split_ascii_whitespace().peekable();

        let action = Action::from(parts.next());
        match action {
            Action::New(_)
            | Action::ClearBlank
            | Action::Deselect
            | Action::Tutorial
            | Action::Delete
            | Action::ToggleMetadata
            | Action::SetMode(_) => Ok(action),
            Action::SetMetadata(_) => self.parse_set_metadata(parts),
            Action::Layout(_) => self.parse_layout(parts),
            Action::Duplicate(_) => self.parse_duplicate(parts),
            Action::Translate(_) => self.parse_translate(parts),
            Action::Write(_) => self.parse_save(parts),
            Action::Read(_) => self.parse_load(parts),
            Action::ShowHelp(_) => self.parse_help(parts),
            Action::Export(_, _) => self.parse_export(parts),
            _ => Err(Error::InvalidCommand),
        }
    }

    fn parse_set_metadata(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let mt_str = parts
            .next()
            .ok_or_else(|| Error::execution("No metadata type specified"))?;

        match mt_str {
            "id" => self.parse_set_id(parts),
            "labels" => self.parse_set_labels(parts),
            _ => Err(Error::execution("Invalid metadata type")),
        }
    }

    fn parse_set_id(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let arg = parts.next().unwrap_or("none");
        let mt = MetadataType::parse_id(arg)?;

        Ok(Action::SetMetadata(mt))
    }

    fn parse_set_labels(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let arg = parts.next().unwrap_or("");
        let mt = MetadataType::parse_labels(arg)?;

        Ok(Action::SetMetadata(mt))
    }

    fn parse_layout(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let type_str = parts.next().unwrap_or("none");

        match Layout::from(type_str) {
            Layout::Random => Ok(Action::Layout(Layout::Random)),
            Layout::Column(_, _) => {
                let cols = parts
                    .next()
                    .ok_or(Error::InvalidParam("No columns specified"))?
                    .parse::<usize>()
                    .map_err(|_| Error::InvalidParam("Invalid columns value"))?;
                let padding_x = parts
                    .next()
                    .ok_or(Error::InvalidParam("No padding specified"))?
                    .parse::<u16>()
                    .map_err(|_| Error::InvalidParam("Invalid padding X value"))?;
                let padding_y = match parts.next() {
                    None => padding_x,
                    Some(str_y) => str_y
                        .parse::<u16>()
                        .map_err(|_| Error::InvalidParam("Invalid padding Y value"))?,
                };

                if cols == 0 {
                    return Err(Error::InvalidParam("Columns must be positive"));
                }

                if padding_x == 0 {
                    return Err(Error::InvalidParam("Padding must be positive"));
                }

                let padding = (padding_x, padding_y);
                Ok(Action::Layout(Layout::Column(cols, padding)))
            }
            Layout::None => Err(Error::InvalidParam("Invalid layout type")),
        }
    }

    fn parse_duplicate(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let count = parts
            .next()
            .unwrap_or("1")
            .parse::<usize>()
            .map_err(|_| Error::InvalidParam("Invalid count value"))?;
        Ok(Action::Duplicate(count))
    }

    fn parse_translate(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        let x = parts
            .next()
            .ok_or(Error::InvalidParam("No X specified"))?
            .parse::<i32>()
            .map_err(|_| Error::InvalidParam("Invalid X value"))?;
        let y = parts
            .next()
            .ok_or(Error::InvalidParam("No Y specified"))?
            .parse::<i32>()
            .map_err(|_| Error::InvalidParam("Invalid Y value"))?;
        Ok(Action::Translate(Translation::Absolute(x, y, None)))
    }

    fn parse_save(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Write(Some(String::from(path))));
        }

        Ok(Action::Write(None))
    }

    fn parse_load(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Read(String::from(path)));
        }

        Err(Error::InvalidParam("No path specified"))
    }

    fn parse_help(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(topic) = parts.next() {
            if let Some(index) = topic_index(topic) {
                Ok(Action::ShowHelp(index))
            } else {
                Err(Error::execution("Invalid topic"))
            }
        } else {
            Ok(Action::ShowHelp(0))
        }
    }

    fn parse_export(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Export(fio::ExportFormat::default(), String::from(path)));
        }

        Err(Error::InvalidParam("No path specified"))
    }
}
