use crate::common::{Action, ExecuteError};
use crate::components::Translation;
use crate::resources::Mode;
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::SplitAsciiWhitespace;
use strum::IntoEnumIterator;
use termion::event::Key;

const DEFAULT_CMD_CAPACITY: usize = 4096; // coz I said so!
const MAX_HISTORY_ENTRIES: usize = 255; // coz I said so too!

#[derive(Debug)]
pub struct CmdLine {
    cmd: String,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl Default for CmdLine {
    fn default() -> Self {
        CmdLine {
            cmd: String::with_capacity(DEFAULT_CMD_CAPACITY),
            history: Vec::with_capacity(MAX_HISTORY_ENTRIES),
            history_index: None,
        }
    }
}

impl CmdLine {
    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    pub fn input(&mut self, k: Key) -> Result<Action, ExecuteError> {
        let result = match k {
            Key::Esc => Ok(Action::ReverseMode),
            Key::Backspace => self.remove(),
            Key::Char(c) => match c {
                '\n' => self.parse(),
                '\t' => self.auto_complete(),
                _ => self.append(c),
            },
            Key::Up => self.previous(),
            Key::Down => self.next(),
            // TODO: handle somehow
            Key::Left | Key::Right => Ok(Action::None),
            _ => Ok(Action::None),
        };

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

    fn previous(&mut self) -> Result<Action, ExecuteError> {
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

    fn next(&mut self) -> Result<Action, ExecuteError> {
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

    fn append(&mut self, c: char) -> Result<Action, ExecuteError> {
        self.cmd.push(c);

        Ok(Action::None)
    }

    fn remove(&mut self) -> Result<Action, ExecuteError> {
        self.cmd.pop();

        Ok(Action::None)
    }

    fn auto_complete(&mut self) -> Result<Action, ExecuteError> {
        let parts: Vec<&str> = self.cmd.split_ascii_whitespace().collect();

        if parts.len() == 1 {
            if let Some(word) = parts.first() {
                if let Some(cmd) = Self::complete_command(word) {
                    self.cmd = cmd;
                }
            }
        }

        Ok(Action::None)
    }

    fn complete_command(partial: &str) -> Option<String> {
        let word = crate::common::to_ascii_titlecase(partial);

        for c in Action::iter() {
            let cmd_str = c.as_ref();
            if cmd_str.starts_with(&word) {
                return Some(cmd_str.to_ascii_lowercase());
            }
        }

        None
    }

    fn parse(&mut self) -> Result<Action, ExecuteError> {
        let mut parts = self.cmd.split_ascii_whitespace().peekable();

        // quit
        if let Some(cmd) = parts.peek() {
            if *cmd == "quit" {
                return Ok(Action::SetMode(Mode::Quitting));
            }
        }

        // try parsing actions
        self.parse_action(parts)
    }

    fn parse_action(
        &self,
        mut parts: Peekable<SplitAsciiWhitespace>,
    ) -> Result<Action, ExecuteError> {
        if let Some(action) = parts.next() {
            let capitalized = crate::common::to_ascii_titlecase(action);

            for a in Action::iter() {
                if a.as_ref() == capitalized {
                    return match a {
                        Action::Delete => Ok(a),
                        Action::Translate(_) => self.parse_translate(parts),
                        Action::Import(_) => self.parse_import(parts),
                        _ => Err(ExecuteError::InvalidCommand),
                    };
                }
            }
        }

        Err(ExecuteError::InvalidCommand)
    }

    fn parse_translate(
        &self,
        mut parts: Peekable<SplitAsciiWhitespace>,
    ) -> Result<Action, ExecuteError> {
        let x = parts
            .next()
            .ok_or(ExecuteError::InvalidParam("No X specified"))?
            .parse::<i32>()
            .map_err(|_| ExecuteError::InvalidParam("Invalid X value"))?;
        let y = parts
            .next()
            .ok_or(ExecuteError::InvalidParam("No Y specified"))?
            .parse::<i32>()
            .map_err(|_| ExecuteError::InvalidParam("Invalid Y value"))?;
        Ok(Action::Translate(Translation::Absolute(x, y)))
    }

    fn parse_import(
        &self,
        mut parts: Peekable<SplitAsciiWhitespace>,
    ) -> Result<Action, ExecuteError> {
        if let Some(path) = parts.next() {
            return Ok(Action::Import(PathBuf::from(path)));
        }

        Err(ExecuteError::InvalidParam("No path specified"))
    }
}
