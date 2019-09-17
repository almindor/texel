use crate::common::{Command, Action};
use crate::components::Translation;
use strum::IntoEnumIterator;
use std::str::SplitAsciiWhitespace;
use std::iter::Peekable;
use termion::event::Key;

const DEFAULT_CMD_CAPACITY: usize = 4096; // coz I said so!
const MAX_HISTORY_ENTRIES: usize = 255; // coz I said so too!

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteError {
    InvalidCommand,
    InvalidParam(&'static str),
    ExecutionError(&'static str),
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::InvalidCommand => write!(f, "Invalid command"),
            ExecuteError::InvalidParam(p) => write!(f, "Invalid parameter: {}", p),
            ExecuteError::ExecutionError(e) => write!(f, "Error: {}", e),
        }
    }
}

impl From<std::convert::Infallible> for ExecuteError {
    fn from(_: std::convert::Infallible) -> Self {
        ExecuteError::InvalidParam("Infallible error?!?!")
    }
}

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

    pub fn input(&mut self, k: Key) -> Result<Command, ExecuteError> {
        let result = match k {
            Key::Esc => Ok(Command::Cancel),
            Key::Backspace => self.remove(),
            Key::Char(c) => match c {
                '\n' => self.parse(),
                '\t' => self.auto_complete(),
                _ => self.append(c),
            },
            Key::Up => self.previous(),
            Key::Down => self.next(),
            // TODO: handle somehow
            Key::Left | Key::Right => Ok(Command::None),
            _ => Ok(Command::None),
        };

        // flush on anything but Command::None
        if let Ok(Command::None) = &result {
        } else {
            self.flush();
        }

        result
    }

    fn flush(&mut self) {
        self.history.push(self.cmd.clone());
        self.cmd.clear();
    }

    fn previous(&mut self) -> Result<Command, ExecuteError> {
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

        Ok(Command::None)
    }

    fn next(&mut self) -> Result<Command, ExecuteError> {
        if let Some(index) = self.history_index {
            if index < self.history.len() - 1 {
                self.cmd.clone_from(&self.history[index + 1]);
                self.history_index = Some(index + 1);
            } else {
                self.cmd.clear();
                self.history_index = None;
            }
        }

        Ok(Command::None)
    }

    fn append(&mut self, c: char) -> Result<Command, ExecuteError> {
        self.cmd.push(c);

        Ok(Command::None)
    }

    fn remove(&mut self) -> Result<Command, ExecuteError> {
        self.cmd.pop();

        Ok(Command::None)
    }

    fn auto_complete(&mut self) -> Result<Command, ExecuteError> {
        let parts: Vec<&str> = self.cmd.split_ascii_whitespace().collect();

        if parts.len() == 1 {
            if let Some(word) = parts.first() {
                if let Some(cmd) = Self::complete_command(word) {
                    self.cmd = cmd;
                }
            }
        }

        Ok(Command::None)
    }

    fn complete_command(partial: &str) -> Option<String> {
        let word = crate::common::to_ascii_titlecase(partial);

        for c in Command::iter() {
            let cmd_str = c.as_ref();
            if cmd_str.starts_with(&word) {
                return Some(cmd_str.to_ascii_lowercase());
            }
        }

        None
    }

    fn parse(&mut self) -> Result<Command, ExecuteError> {
        let mut parts = self.cmd.split_ascii_whitespace().peekable();

        // check basic global commands
        if let Some(cmd) = parts.peek() {
            let capitalized = crate::common::to_ascii_titlecase(cmd);

            for c in Command::iter() {
                if c.as_ref() == capitalized {
                    return match c {
                        Command::Engage | Command::Clear | Command::Cancel | Command::Quit => Ok(c),
                        _ => Err(ExecuteError::InvalidCommand),
                    }
                }
            }
        }

        // try parsing actions
        self.parse_action(parts)
    }

    fn parse_action(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Command, ExecuteError> {
        if let Some(action) = parts.next() {
            let capitalized = crate::common::to_ascii_titlecase(action);

            for a in Action::iter() {
                if a.as_ref() == capitalized {
                    return match a {
                        Action::Delete => Ok(Command::Perform(a)),
                        Action::Translate(_) => self.parse_translate(parts),
                        _ => Err(ExecuteError::InvalidCommand),
                    }
                }
            }
        }

        Err(ExecuteError::InvalidCommand)
    }

    fn parse_translate(&self, mut parts: Peekable<SplitAsciiWhitespace>) -> Result<Command, ExecuteError> {
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
        Ok(Command::Perform(Action::Translate(Translation::Absolute(
            x, y,
        ))))
    }
}
