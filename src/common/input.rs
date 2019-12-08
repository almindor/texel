use crate::common::{ClipboardOp, Mode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use termion::event::{Event as TEvent, Key};
use texel_types::{ColorMode, Position2D, SymbolStyle, Which};

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
    Mode(Mode),
    ApplyStyle(SymbolStyle),
    EditPalette(usize), // index of symbol/color, 0x0-0xF as usize <0, 16)
    ApplyColor(ColorMode),
    SelectObject(Which<Position2D>, bool), // sticky boolean
    SelectFrame(Which<usize>),
    Clipboard(ClipboardOp),
    NewFrame,
    DeleteFrame,
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

        map.insert(':', Event::Mode(Mode::Command));
        map.insert('e', Event::Mode(Mode::Edit));
        map.insert('i', Event::Mode(Mode::Write));
        // 1-0 + A,B,C,D,E,F (HEX) are symbol overrides
        map.insert('!', Event::EditPalette(0));
        map.insert('@', Event::EditPalette(1));
        map.insert('#', Event::EditPalette(2));
        map.insert('$', Event::EditPalette(3));
        map.insert('%', Event::EditPalette(4));
        map.insert('^', Event::EditPalette(5));
        map.insert('&', Event::EditPalette(6));
        map.insert('*', Event::EditPalette(7));
        map.insert('(', Event::EditPalette(8));
        map.insert(')', Event::EditPalette(9));
        map.insert('A', Event::EditPalette(10));
        map.insert('B', Event::EditPalette(11));
        map.insert('C', Event::EditPalette(12));
        map.insert('D', Event::EditPalette(13));
        map.insert('E', Event::EditPalette(14));
        map.insert('F', Event::EditPalette(15));

        map.insert('Z', Event::Mode(Mode::Color(ColorMode::Fg)));
        map.insert('X', Event::Mode(Mode::Color(ColorMode::Bg)));

        map.insert('z', Event::ApplyColor(ColorMode::Fg));
        map.insert('x', Event::ApplyColor(ColorMode::Bg));

        map.insert('q', Event::ApplyStyle(SymbolStyle::Bold));
        map.insert('Q', Event::ApplyStyle(SymbolStyle::Italic));
        map.insert('w', Event::ApplyStyle(SymbolStyle::Underline));

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

        map.insert(']', Event::SelectFrame(Which::Next));
        map.insert('[', Event::SelectFrame(Which::Previous));
        map.insert('+', Event::NewFrame);
        map.insert('_', Event::DeleteFrame);

        map.insert('u', Event::Undo);
        map.insert('U', Event::Redo);

        map.insert('y', Event::Clipboard(ClipboardOp::Copy));
        map.insert('Y', Event::Clipboard(ClipboardOp::Cut));
        map.insert('p', Event::Clipboard(ClipboardOp::Paste));

        map.insert('n', Event::NewObject);

        map.insert('\n', Event::Confirm);
        map.insert('\t', Event::SelectObject(Which::Next, false));

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
        result.map.insert(TEvent::Key(Key::Right), Event::ArrowRight);
        result.map.insert(TEvent::Key(Key::Up), Event::ArrowUp);
        result.map.insert(TEvent::Key(Key::Down), Event::ArrowDown);
        result.map.insert(TEvent::Key(Key::Delete), Event::Delete);
        result.map.insert(TEvent::Key(Key::Backspace), Event::Backspace);
        result
            .map
            .insert(TEvent::Key(Key::BackTab), Event::SelectObject(Which::Next, true));

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
