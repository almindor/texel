use crate::common::{path_base, Action, Error};
use crate::components::Translation;
use crate::resources::{Loader};
use std::iter::Peekable;
use std::str::SplitAsciiWhitespace;
use termion::event::Key;

mod auto_complete;

use auto_complete::FileComplete;

const DEFAULT_CMD_CAPACITY: usize = 4096; // coz I said so!
const MAX_HISTORY_ENTRIES: usize = 255; // coz I said so too!

#[derive(Debug)]
pub struct CmdLine {
    cmd: String,
    history: Vec<String>,
    history_index: Option<usize>,
    file_complete: FileComplete,
}

impl Default for CmdLine {
    fn default() -> Self {
        CmdLine {
            cmd: String::with_capacity(DEFAULT_CMD_CAPACITY),
            history: Vec::with_capacity(MAX_HISTORY_ENTRIES),
            history_index: None,
            file_complete: FileComplete::new(),
        }
    }
}

impl CmdLine {
    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn input(&mut self, k: Key) -> Result<Action, Error> {
        let mut clear_ac = true;
        let result = match k {
            Key::Esc => Ok(Action::ReverseMode),
            Key::Backspace => self.remove(),
            Key::Char(c) => match c {
                '\n' => self.parse(),
                '\t' => {
                    clear_ac = false;
                    self.auto_complete()
                }
                _ => self.append(c),
            },
            Key::Up => self.previous(),
            Key::Down => self.next(),
            // TODO: handle somehow
            Key::Left | Key::Right => Ok(Action::None),
            _ => Ok(Action::None),
        };

        if clear_ac {
            self.file_complete.clear();
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
        self.cmd.clear();
    }

    fn previous(&mut self) -> Result<Action, Error> {
        if self.history.len() > 0 {
            if self.history_index.is_none() {
                self.history_index = Some(self.history.len() - 1);
            }

            if let Some(index) = self.history_index {
                self.cmd.clone_from(&self.history[index]);
                if index > 0 {
                    self.history_index = Some(index - 1);
                }
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
        }

        Ok(Action::None)
    }

    fn append(&mut self, c: char) -> Result<Action, Error> {
        self.cmd.push(c);

        Ok(Action::None)
    }

    fn remove(&mut self) -> Result<Action, Error> {
        self.cmd.pop();

        Ok(Action::None)
    }

    fn auto_complete(&mut self) -> Result<Action, Error> {
        let parts: Vec<&str> = self.cmd.split_ascii_whitespace().collect();

        if parts.len() == 1 {
            if let Some(word) = parts.first() {
                if let Some(cmd) = Action::complete_word(word) {
                    self.cmd = path_base(&self.cmd) + cmd;
                }
            }
        } else if let Some(cmd) = parts.first() {
            // if we have something here, and count != 1 parts.count() must be >= 1
            let completed = match *cmd {
                "import" | "load" | "save" => {
                    self.file_complete.with_path(parts.last().unwrap_or(&"."))?
                } // parts.last() is safe here
                _ => None,
            };

            if let Some(path) = completed {
                self.cmd = String::from(*cmd) + " " + &path;
            }
        }

        Ok(Action::None)
    }

    fn parse(&mut self) -> Result<Action, Error> {
        let mut parts = self.cmd.split_ascii_whitespace().peekable();

        let action = Action::from(parts.next());
        match action {
            Action::Delete | Action::Deselect | Action::SetMode(_) => Ok(action),
            Action::Translate(_) => self.parse_translate(parts),
            Action::Import(_) => self.parse_import(parts),
            Action::Save(_) => self.parse_save(parts),
            Action::Load(_) => self.parse_load(parts),
            _ => Err(Error::InvalidCommand),
        }
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

    fn parse_import(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Import(Loader::from_txt_file(path)?));
        }

        Err(Error::InvalidParam("No path specified"))
    }

    fn parse_save(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Save(String::from(path)));
        }

        Err(Error::InvalidParam("No path specified"))
    }

    fn parse_load(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Action, Error> {
        if let Some(path) = parts.next() {
            return Ok(Action::Load(String::from(path)));
        }

        Err(Error::InvalidParam("No path specified"))
    }
}
