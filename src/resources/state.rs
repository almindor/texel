use crate::common::{Action, Clipboard, Error, InputEvent, Mode, Scene};
use crate::components::Position2D;
use std::collections::VecDeque;
use texel_types::ColorMode;

const HISTORY_CAPACITY: usize = 20usize;

// snapshot of editor state
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
    pub scene: Scene,
    pub selections: Vec<usize>,
}

impl From<Scene> for Snapshot {
    fn from(scene: Scene) -> Self {
        Snapshot {
            scene,
            selections: Vec::new(),
        }
    }
}

impl From<(Scene, Vec<usize>)> for Snapshot {
    fn from(data: (Scene, Vec<usize>)) -> Self {
        Snapshot {
            scene: data.0,
            selections: data.1,
        }
    }
}

// #[derive(Debug)]
pub struct State {
    // state
    error: Option<Error>,
    events: VecDeque<InputEvent>, // (raw, Option<mapping>)
    actions: VecDeque<Action>,
    modes: VecDeque<Mode>,
    history: VecDeque<Snapshot>, // scene + list of selected indexes
    history_index: usize,
    selected_color: (u8, u8),
    save_state: (Option<String>, usize, usize), // save file path, changes, change "start" index
    // TODO: refactor these off?
    offset: Position2D, // viewport "offset"
    pub dirty: bool,
    pub clipboard: Clipboard,
    pub cursor: Position2D,
    pub mouse_entry: Position2D, // previous mouse position for dragging
    pub show_meta: bool,
}

impl Default for State {
    fn default() -> Self {
        let mut result = State {
            // state
            error: None,
            events: VecDeque::with_capacity(10),
            actions: VecDeque::with_capacity(10),
            modes: VecDeque::with_capacity(5),
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            history_index: 0usize,
            selected_color: (texel_types::DEFAULT_BG_U8, texel_types::DEFAULT_FG_U8),
            save_state: (None, 0, 0),
            // others
            dirty: false,
            clipboard: Clipboard::Empty,
            cursor: Position2D::default(),
            offset: Position2D::default(),
            mouse_entry: Position2D::default(),
            show_meta: false,
        };

        result.modes.push_back(Mode::default()); // there is always a mode!
        result.history.push_back(Snapshot::default()); // empty

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

    pub fn swap_color(&mut self) -> bool {
        std::mem::swap(&mut self.selected_color.0, &mut self.selected_color.1);

        false
    }

    // hacky way to keep help + command_during_help from offsetting the viewport
    fn offset_for_mode(&self, mode: Mode) -> Position2D {
        match mode {
            Mode::Help(_) => Position2D::default(),
            Mode::Command => {
                if let Some(prev_mode) = self.previous_mode() {
                    self.offset_for_mode(prev_mode)
                } else {
                    self.offset
                }
            }
            _ => self.offset,
        }
    }

    pub fn offset(&self) -> Position2D {
        self.offset_for_mode(self.mode())
    }

    pub fn offset_mut(&mut self) -> &mut Position2D {
        &mut self.offset
    }

    pub fn set_offset(&mut self, offset: Position2D) {
        self.offset = offset;
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

    pub fn unsaved_changes(&self) -> bool {
        self.save_state.1 > 0 && self.history_index != self.save_state.2
    }

    pub fn set_mode(&mut self, mode: Mode) -> bool {
        if self.mode() != mode {
            self.push_action(Action::ClearError); // clear errors on mode changes
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

    pub fn previous_mode(&self) -> Option<Mode> {
        if self.modes.len() > 1 {
            self.modes.get(self.modes.len() - 2).cloned()
        } else {
            None
        }
    }

    pub fn show_help(&self) -> Option<usize> {
        if self.modes.len() > 1 {
            if let Mode::Help(index) = self.mode() {
                return Some(index);
            } else if let Some(previous_mode) = self.previous_mode() {
                if let Mode::Help(index) = previous_mode {
                    return Some(index);
                }
            }
            None
        } else {
            None
        }
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

    pub fn filename(&self) -> &str {
        self.save_state.0.as_deref().unwrap_or("NONE")
    }

    pub fn saved(&mut self, path: String) -> bool {
        self.save_state = (Some(path), 0, self.history_index);

        false
    }

    pub fn clear_changes(&mut self) -> bool {
        self.save_state.1 = 0; // keep filename
        self.save_state.2 = self.history_index;

        false
    }

    pub fn reset_save_file(&mut self) -> bool {
        self.save_state = (None, 0, 0);

        false
    }

    pub fn quitting(&self) -> bool {
        matches!(self.mode(), Mode::Quitting(_))
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

    // returns if we need to preserve anything
    // this includes non-dirty selection updates if history.len() == 1
    pub fn needs_preserving(&self) -> bool {
        self.dirty || self.history.len() == 1
    }

    // resets history to start with this scene
    pub fn clear_history(&mut self, scene: Scene) {
        self.history.clear();
        self.history.push_back(Snapshot::from(scene));
        self.history_index = 0;
        self.dirty = false;
    }

    // the minimalist in me screams at this, but doing a delta with
    // multi-selections and absolute location selections (mouse) is a
    // major PITA with ECS context, saving the whole thing for undo/redo is
    // just so much easier to do. Given it's ascii we're only taking kilobytes of
    // space under typical usage
    pub fn push_history(&mut self, scene: Scene, selections: Vec<usize>) {
        if !self.dirty {
            // non-dirtying change in initial setup, refresh selections
            if self.history.len() == 1 && !selections.is_empty() {
                self.history[0].scene = scene; // ordering can change, preserve it
                self.history[0].selections = selections;
            }

            return;
        }

        if self.history.len() >= HISTORY_CAPACITY {
            self.history.pop_front();
        }

        if !self.history.is_empty() && self.history_index != self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }

        self.history.push_back(Snapshot::from((scene, selections)));
        let next_index = self.history.len() - 1;

        self.history_index = next_index;
        self.dirty = false;

        self.save_state.1 += 1;
    }

    pub fn undo(&mut self) -> Option<Snapshot> {
        if self.history_index == 0 {
            return None;
        }

        self.history_index -= 1;
        if let Some(value) = self.history.get(self.history_index) {
            self.save_state.1 += 1;
            return Some(value.clone());
        }

        None
    }

    pub fn redo(&mut self) -> Option<Snapshot> {
        if self.history.is_empty() || self.history_index >= self.history.len() - 1 {
            return None;
        }

        self.history_index += 1;
        if let Some(value) = self.history.get(self.history_index) {
            self.save_state.1 += 1;
            return Some(value.clone());
        }

        None
    }
}
