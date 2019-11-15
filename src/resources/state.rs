use crate::common::{Action, Error, InputEvent, Scene};
use crate::components::Position2D;
use crate::resources::{ColorMode, ColorPalette};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Object,
    Color(ColorMode),
    SelectColor(usize),  // index for which color 0 -> 16 (0x0 to 0xF)
    SelectSymbol(usize), // index for which symbol 0 -> 16 (0x0 to 0xF)
    Edit,
    Command,
    Quitting(bool), // true for force quit
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Object
    }
}

impl Mode {
    pub fn to_str(&self) -> &'static str {
        match self {
            Mode::Object => "OBJECT",
            Mode::Color(ColorMode::Fg) => "COLOR[FG]",
            Mode::Color(ColorMode::Bg) => "COLOR[BG]",
            Mode::SelectColor(_) => "COLOR[SET]", // TODO: construct static numbered index
            Mode::SelectSymbol(_) => "SYMBOL[SET]", // TODO: construct static numbered index
            Mode::Edit => "EDIT",
            Mode::Command => "COMMAND",
            Mode::Quitting(_) => "QUITTING",
        }
    }
}

const HISTORY_CAPACITY: usize = 20usize;

#[derive(Debug)]
pub struct State {
    error: Option<Error>,
    events: VecDeque<InputEvent>, // (raw, Option<mapping>)
    actions: VecDeque<Action>,
    modes: VecDeque<Mode>,
    history: VecDeque<Scene>,
    history_index: usize,
    selected_color: (u8, u8),
    save_state: (Option<String>, usize),
    dirty: bool,
    pub cursor: Position2D,
}

impl Default for State {
    fn default() -> Self {
        let mut result = State {
            error: None,
            events: VecDeque::with_capacity(10),
            actions: VecDeque::with_capacity(10),
            modes: VecDeque::with_capacity(5),
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            history_index: 0usize,
            selected_color: (ColorPalette::default_bg_u8(), ColorPalette::default_fg_u8()),
            save_state: (None, 0),
            dirty: false,
            cursor: Position2D::default(),
        };

        result.modes.push_back(Mode::default()); // there is always a mode!
        result.push_history(Scene::default()); // there is always a default scene
        result.save_state.1 = 0; // push_history will bump this but it doesn't count

        result
    }
}

impl State {
    pub fn error(&self) -> &Option<Error> {
        &self.error
    }

    pub fn color(&self, cm: ColorMode) -> u8 {
        match cm {
            ColorMode::Bg => self.selected_color.0,
            ColorMode::Fg => self.selected_color.1,
        }
    }

    pub fn set_color(&mut self, color: u8, cm: ColorMode) {
        match cm {
            ColorMode::Bg => self.selected_color.0 = color,
            ColorMode::Fg => self.selected_color.1 = color,
        }
    }

    // returns bool so we can easily chain to "changed"
    // in action handler, bit of a hack
    pub fn set_error(&mut self, error: Error) -> bool {
        self.error = Some(error);
        false
    }

    pub fn clear_error(&mut self) -> bool {
        self.error = None;
        false
    }

    pub fn mode(&self) -> Mode {
        *self.modes.back().unwrap()
    }

    pub fn set_mode(&mut self, mode: Mode) -> bool {
        if self.mode() != mode {
            if mode == Mode::Quitting(false) && self.save_state.1 > 0 {
                self.set_error(Error::execution("Unsaved changes, use q! to quit without saving"));
                return false;
            }

            self.modes.push_back(mode);
            return true;
        }

        false
    }

    pub fn reverse_mode(&mut self) -> bool {
        if self.modes.len() > 1 {
            self.modes.pop_back();
            return true;
        }

        false
    }

    // needs to clone because we need to keep the option for saved() if
    // everything went fine and scene was saved, also the temporary new_path
    // cannot be referenced out because it has no place to live in
    pub fn save_file(&self, new_path: &Option<String>) -> Result<String, Error> {
        if let Some(path) = new_path {
            Ok(path.clone())
        } else if let Some(path) = &self.save_state.0 {
            Ok(path.clone())
        } else {
            Err(Error::execution("No file path specified"))
        }
    }

    pub fn saved(&mut self, path: Option<String>) -> bool {
        self.save_state = (path, 0);
        true
    }

    pub fn quitting(&self) -> bool {
        match self.mode() {
            Mode::Quitting(_) => true,
            _ => false,
        }
    }

    pub fn push_event(&mut self, event: InputEvent) {
        self.events.push_back(event)
    }

    pub fn pop_event(&mut self) -> Option<InputEvent> {
        self.events.pop_front()
    }

    pub fn push_action(&mut self, action: Action) {
        if !action.is_some() {
            return;
        }

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

        self.save_state.1 += 1;
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
