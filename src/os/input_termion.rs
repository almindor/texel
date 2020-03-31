use crate::common::{CharMap, Event, InputEvent, Mode, ModesCharMap, MoveMeta};
use std::collections::HashMap;
use std::io::stdin;
use termion::event::{Event as TEvent, Key};
use termion::input::TermRead;
use texel_types::Which;

type RawMap = HashMap<TEvent, Event>;

pub struct InputSource {
    mode_maps: Vec<RawMap>,
}

impl InputSource {
    pub fn next_event(&self, mode: Mode) -> InputEvent {
        let map = self
            .mode_maps
            .get(mode.index())
            .unwrap_or_else(|| panic!("Mode map not found"));

        match stdin().events().next() {
            None => panic!("Error on input"),
            Some(result) => self.map_input(result.unwrap(), map),
        }
    }

    fn map_input(&self, raw_event: TEvent, map: &RawMap) -> InputEvent {
        let mapped = map.get(&raw_event).copied().unwrap_or(Event::None);

        match raw_event {
            TEvent::Key(Key::Char(c)) => (mapped, Some(c)),
            _ => (mapped, None),
        }
    }
}

impl From<ModesCharMap> for InputSource {
    fn from(cm: ModesCharMap) -> Self {
        let mut result = InputSource {
            mode_maps: Vec::with_capacity(Mode::count()),
        };

        let defaults = default_map(cm.all_modes());

        // modes full maps
        for o in cm.overrides() {
            let mut mode_map = defaults.clone();
            char_map_to_raw_map(o, &mut mode_map); // override specifics
            result.mode_maps.push(mode_map);
        }

        result
    }
}

fn default_map(cm: &CharMap) -> RawMap {
    let mut result = RawMap::new();
    char_map_to_raw_map(cm, &mut result);

    // meta-key defaults
    result.insert(TEvent::Key(Key::Esc), Event::Cancel);
    result.insert(TEvent::Key(Key::Left), Event::ArrowLeft);
    result.insert(TEvent::Key(Key::Right), Event::ArrowRight);
    result.insert(TEvent::Key(Key::Up), Event::ArrowUp);
    result.insert(TEvent::Key(Key::Down), Event::ArrowDown);
    result.insert(TEvent::Key(Key::Delete), Event::Delete);
    result.insert(TEvent::Key(Key::Backspace), Event::Backspace);
    result.insert(TEvent::Key(Key::Ctrl('a')), Event::SelectObject(Which::All, false));

    result.insert(TEvent::Key(Key::Ctrl('h')), Event::Left(MoveMeta::Alternative));
    result.insert(TEvent::Key(Key::Ctrl('j')), Event::Right(MoveMeta::Alternative));
    result.insert(TEvent::Key(Key::Ctrl('k')), Event::Up(MoveMeta::Alternative));
    result.insert(TEvent::Key(Key::Ctrl('l')), Event::Down(MoveMeta::Alternative));
    result.insert(TEvent::Key(Key::BackTab), Event::SelectObject(Which::Next, true));

    result
}

fn char_map_to_raw_map(cm: &CharMap, raw_map: &mut RawMap) {
    for (c, v) in &cm.0 {
        let new_key = TEvent::Key(Key::Char(*c));
        raw_map.insert(new_key, *v);
    }
}
