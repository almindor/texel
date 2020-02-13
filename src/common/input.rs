use crate::common::{ClipboardOp, Mode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use texel_types::{ColorMode, Position2D, SymbolStyle, Which};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveMeta {
    Relative,
    ToEdge,
    Alternative,
}

impl Default for MoveMeta {
    fn default() -> Self {
        Self::Relative
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    None,
    Cancel,
    Confirm,
    Left(MoveMeta),
    Up(MoveMeta),
    Right(MoveMeta),
    Down(MoveMeta),
    Above,
    Below,
    Undo,
    Redo,
    Mode(Mode),
    ApplyStyle(SymbolStyle),
    SelectPalette(usize), // index of symbol/color/bookmark, 0x0-0xF as usize <0, 16)
    EditPalette(usize),   // index of symbol/color/bookmark, 0x0-0xF as usize <0, 16)
    ApplyColor(ColorMode),
    SelectObject(Which<Position2D>, bool), // sticky boolean
    SelectRegion,
    SelectFrame(Which<usize>),
    Clipboard(ClipboardOp),
    ToggleMetadata,
    NewFrame,
    DeleteFrame,
    NewObject,
    Duplicate(usize), // count
    Deselect,
    // "meta" keys
    Delete,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    // terminal events
    Resize,
}

impl Default for Event {
    fn default() -> Self {
        Event::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharMap(pub HashMap<char, Event>);

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

        map.insert('1', Event::SelectPalette(0));
        map.insert('2', Event::SelectPalette(1));
        map.insert('3', Event::SelectPalette(2));
        map.insert('4', Event::SelectPalette(3));
        map.insert('5', Event::SelectPalette(4));
        map.insert('6', Event::SelectPalette(5));
        map.insert('7', Event::SelectPalette(6));
        map.insert('8', Event::SelectPalette(7));
        map.insert('9', Event::SelectPalette(8));
        map.insert('0', Event::SelectPalette(9));
        map.insert('a', Event::SelectPalette(10));
        map.insert('b', Event::SelectPalette(11));
        map.insert('c', Event::SelectPalette(12));
        map.insert('d', Event::SelectPalette(13));
        map.insert('e', Event::SelectPalette(14));
        map.insert('f', Event::SelectPalette(15));

        map.insert('Z', Event::Mode(Mode::Color(ColorMode::Fg)));
        map.insert('X', Event::Mode(Mode::Color(ColorMode::Bg)));

        map.insert('z', Event::ApplyColor(ColorMode::Fg));
        map.insert('x', Event::ApplyColor(ColorMode::Bg));

        map.insert('q', Event::ApplyStyle(SymbolStyle::Bold));
        map.insert('Q', Event::ApplyStyle(SymbolStyle::Italic));
        map.insert('w', Event::ApplyStyle(SymbolStyle::Underline));

        map.insert('h', Event::Left(MoveMeta::Relative));
        map.insert('j', Event::Down(MoveMeta::Relative));
        map.insert('k', Event::Up(MoveMeta::Relative));
        map.insert('l', Event::Right(MoveMeta::Relative));

        map.insert('H', Event::Left(MoveMeta::ToEdge));
        map.insert('J', Event::Down(MoveMeta::ToEdge));
        map.insert('K', Event::Up(MoveMeta::ToEdge));
        map.insert('L', Event::Right(MoveMeta::ToEdge));

        map.insert('-', Event::Above);
        map.insert('=', Event::Below);

        map.insert(']', Event::SelectFrame(Which::Next));
        map.insert('[', Event::SelectFrame(Which::Previous));
        map.insert('}', Event::NewFrame);
        map.insert('{', Event::DeleteFrame);

        map.insert('u', Event::Undo);
        map.insert('U', Event::Redo);

        map.insert('y', Event::Clipboard(ClipboardOp::Copy));
        map.insert('Y', Event::Clipboard(ClipboardOp::Cut));
        map.insert('p', Event::Clipboard(ClipboardOp::Paste));

        map.insert('m', Event::ToggleMetadata);

        map.insert('n', Event::NewObject);
        map.insert('t', Event::Duplicate(1));

        map.insert('\n', Event::Confirm);
        map.insert('\t', Event::SelectObject(Which::Next, false));
        map.insert('v', Event::SelectRegion);

        CharMap(map)
    }
}

pub type InputEvent = (Event, Option<char>);
