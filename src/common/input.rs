use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use termion::event::{Event as TEvent, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    None,
    Cancel,
    Confirm,
    Left,
    Up,
    Right,
    Above,
    Below,
    Down,
    Undo,
    Redo,
    LeftEdge,
    UpEdge,
    RightEdge,
    DownEdge,
    ModeCmd,
    ModeEdit,
    ModeColorFG,
    ModeColorBG,
    ApplyColorFG,
    ApplyColorBG,
    Next,
    NextWith,
    NewObject,
    // "meta" keys
    Delete,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

impl Default for Event {
    fn default() -> Self {
        Event::None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharMap {
    map: HashMap<char, Event>,
}

impl Default for CharMap {
    fn default() -> Self {
        let mut map = HashMap::with_capacity(30);

        map.insert(':', Event::ModeCmd);
        map.insert('e', Event::ModeEdit);

        map.insert('F', Event::ModeColorFG);
        map.insert('B', Event::ModeColorBG);

        map.insert('h', Event::Left);
        map.insert('j', Event::Down);
        map.insert('k', Event::Up);
        map.insert('l', Event::Right);

        map.insert('H', Event::LeftEdge);
        map.insert('J', Event::DownEdge);
        map.insert('K', Event::UpEdge);
        map.insert('L', Event::RightEdge);

        map.insert('-', Event::Above);
        map.insert('=', Event::Below);

        map.insert('u', Event::Undo);
        map.insert('U', Event::Redo);

        map.insert('n', Event::NewObject);

        map.insert('\n', Event::Confirm);
        map.insert('\t', Event::Next);

        CharMap { map }
    }
}

pub type InputEvent = (Event, Option<char>);

#[derive(Debug)]
pub struct InputMap {
    map: HashMap<TEvent, Event>,
}

impl From<CharMap> for InputMap {
    fn from(cm: CharMap) -> Self {
        let mut result = InputMap {
            map: HashMap::with_capacity(cm.map.capacity()),
        };

        for (c, v) in cm.map {
            let new_key = TEvent::Key(Key::Char(c));
            result.map.insert(new_key, v);
        }

        // meta-key defaults
        result.map.insert(TEvent::Key(Key::Esc), Event::Cancel);
        result.map.insert(TEvent::Key(Key::Left), Event::ArrowLeft);
        result
            .map
            .insert(TEvent::Key(Key::Right), Event::ArrowRight);
        result.map.insert(TEvent::Key(Key::Up), Event::ArrowUp);
        result.map.insert(TEvent::Key(Key::Down), Event::ArrowDown);
        result.map.insert(TEvent::Key(Key::Delete), Event::Delete);
        result
            .map
            .insert(TEvent::Key(Key::Backspace), Event::Backspace);
        result
            .map
            .insert(TEvent::Unsupported(vec![27, 91, 90]), Event::NextWith);

        result
    }
}

impl InputMap {
    pub fn map_input(&self, raw_event: TEvent) -> InputEvent {
        let mapped = self.map.get(&raw_event).copied().unwrap_or(Event::None);

        match raw_event {
            TEvent::Key(Key::Char(c)) => (mapped, Some(c)),
            _ => (mapped, None),
        }
    }
}
