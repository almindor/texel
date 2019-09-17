use crate::common::Action;
use crate::resources::ExecuteError;
use std::collections::VecDeque;
use termion::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Command,
    Object,
    Immediate,
    Quitting,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object
    }
}

#[derive(Default, Debug)]
pub struct State {
    pub error: Option<ExecuteError>,
    events: VecDeque<Event>,
    actions: VecDeque<Action>,
    mode: Mode,
    prev_mode: Mode,
}

impl State {
    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.prev_mode = self.mode;
            self.mode = mode;
        }

        // clear error
        self.error = None
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
