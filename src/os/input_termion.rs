use crate::common::{CharMap, Event, InputEvent, MoveMeta};
use std::collections::HashMap;
use std::io::{stdin, Stdin};
use termion::event::{Event as TEvent, Key};
use termion::input::{Events, TermRead};
use texel_types::Which;

type RawMap = HashMap<TEvent, Event>;

struct MappedIter<'a> {
    source: Events<Stdin>,
    map: &'a RawMap,
}

impl MappedIter<'_> {
    fn map_input(&self, raw_event: TEvent) -> InputEvent {
        let mapped = self.map.get(&raw_event).copied().unwrap_or(Event::None);

        match raw_event {
            TEvent::Key(Key::Char(c)) => (mapped, Some(c)),
            _ => (mapped, None),
        }
    }
}

impl Iterator for MappedIter<'_> {
    type Item = InputEvent;

    fn next(&mut self) -> Option<InputEvent> {
        match self.source.next() {
            None => None,
            Some(result) => Some(self.map_input(result.unwrap())),
        }
    }
}

#[derive(Debug)]
pub struct InputSource {
    map: RawMap,
}

impl InputSource {
    pub fn events(&self) -> impl Iterator<Item = InputEvent> + '_ {
        MappedIter {
            source: stdin().events(),
            map: &self.map,
        }
    }
}

impl From<CharMap> for InputSource {
    fn from(cm: CharMap) -> Self {
        let mut result = InputSource {
            map: HashMap::with_capacity(cm.0.capacity()),
        };

        for (c, v) in cm.0 {
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
            .insert(TEvent::Key(Key::Ctrl('h')), Event::Left(MoveMeta::Alternative));
        result
            .map
            .insert(TEvent::Key(Key::Ctrl('j')), Event::Right(MoveMeta::Alternative));
        result
            .map
            .insert(TEvent::Key(Key::Ctrl('k')), Event::Up(MoveMeta::Alternative));
        result
            .map
            .insert(TEvent::Key(Key::Ctrl('l')), Event::Down(MoveMeta::Alternative));
        result
            .map
            .insert(TEvent::Key(Key::BackTab), Event::SelectObject(Which::Next, true));

        result
    }
}
