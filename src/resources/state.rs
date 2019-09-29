use crate::common::{Action, Error, Scene};
use std::collections::VecDeque;
use termion::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Object,
    Color,
    Immediate,
    Command,
    Quitting(bool), // true for force quit
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object
    }
}

const HISTORY_CAPACITY: usize = 20usize;

#[derive(Debug)]
pub struct State {
    error: Option<Error>,
    events: VecDeque<Event>,
    actions: VecDeque<Action>,
    mode: Mode,
    prev_mode: Mode,
    history: VecDeque<Scene>,
    history_index: usize,
    dirty: bool,
}

impl Default for State {
    fn default() -> Self {
        let mut result = State {
            error: None,
            events: VecDeque::with_capacity(10usize),
            actions: VecDeque::with_capacity(10usize),
            mode: Mode::default(),
            prev_mode: Mode::default(),
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            history_index: 0usize,
            dirty: true,
        };

        result.push_history(Scene::default());

        result
    }
}

impl State {
    pub fn error(&self) -> &Option<Error> {
        &self.error
    }

    // returns bool so we can easily chain to "changed"
    // in action handler, bit of a hack
    pub fn set_error(&mut self, error: Option<Error>) -> bool {
        self.error = error;
        false
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) -> bool {
        if self.mode != mode {
            self.prev_mode = self.mode;
            self.mode = mode;
            return true;
        }

        false
    }

    pub fn quitting(&self) -> bool {
        match self.mode {
            Mode::Quitting(_) => true,
            _ => false,
        }
    }

    pub fn reverse_mode(&mut self) -> bool {
        if self.mode != self.prev_mode {
            self.mode = self.prev_mode;
            return true;
        }

        false
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

    pub fn dirty(&mut self) {
        self.dirty = true;
    }

    // the minimalist in me screams at this, but doing a delta with
    // multi-selections and absolute location selections (mouse) is a
    // major PITA with ECS context, saving the whole thing for undo/redo is
    // just so much easier to do. Given it's ascii we're only taking kilobytes of
    // space under typical usage
    pub fn push_history(&mut self, scene: Scene) {
        if !self.dirty {
            return;
        }

        if self.history.len() >= HISTORY_CAPACITY {
            self.history.pop_front();
        }

        if self.history.len() > 0 && self.history_index != self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }

        self.history.push_back(scene);
        let next_index = self.history.len() - 1;

        self.history_index = next_index;
        self.dirty = false;
    }

    pub fn undo(&mut self) -> Option<Scene> {
        if self.history_index == 0 {
            return None;
        }

        self.history_index -= 1;
        if let Some(scene) = self.history.get(self.history_index) {
            return Some(scene.clone());
        }

        None
    }

    pub fn redo(&mut self) -> Option<Scene> {
        if self.history.len() == 0 || self.history_index >= self.history.len() - 1 {
            return None;
        }

        self.history_index += 1;
        if let Some(scene) = self.history.get(self.history_index) {
            return Some(scene.clone());
        }

        None
    }
}
