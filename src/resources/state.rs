use crate::common::{Action, Error};
use std::collections::VecDeque;
use termion::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Object,
    Immediate,
    Command,
    Quitting(bool), // true for --force
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object
    }
}

#[derive(Default, Debug)]
pub struct State {
    error: Option<Error>,
    events: VecDeque<Event>,
    actions: VecDeque<Action>,
    mode: Mode,
    prev_mode: Mode,
}

impl State {
    pub fn error(&self) -> &Option<Error> {
        &self.error
    }

    pub fn set_error(&mut self, error: Option<Error>) {
        self.error = error;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.prev_mode = self.mode;
            self.mode = mode;
        }
    }

    pub fn quitting(&self) -> bool {
        match self.mode {
            Mode::Quitting(_) => true,
            _ => false,
        }
    }

    pub fn reverse_mode(&mut self) {
        self.mode = self.prev_mode;
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event)
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn push_action(&mut self, action: Action) {
        self.actions.push_back(action);
    }

    pub fn pop_action(&mut self) -> Option<Action> {
        self.actions.pop_front()
    }
}
