use crate::common::{CharMap, Event, InputEvent, MoveMeta};
use crossterm::event::{read, Event as TEvent, KeyCode as Key, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use texel_types::Which;

type RawMap = HashMap<TEvent, Event>;

struct MappedIter<'a> {
    map: &'a RawMap,
}

impl MappedIter<'_> {
    fn map_input(&self, raw_event: TEvent) -> InputEvent {
        let mapped = self.map.get(&raw_event).copied().unwrap_or_else(|| match raw_event {
            TEvent::Resize(_, _) => Event::Resize,
            _ => Event::None,
        });

        match raw_event {
            TEvent::Key(key_event) => match key_event.code {
                Key::Char(c) => (mapped, Some(c)),
                _ => (mapped, None),
            },
            _ => (mapped, None),
        }
    }
}

impl Iterator for MappedIter<'_> {
    type Item = InputEvent;

    fn next(&mut self) -> Option<InputEvent> {
        match read() {
            Err(err) => panic!(err), // TODO: maybe find a better handler?
            Ok(result) => Some(self.map_input(result)),
        }
    }
}

#[derive(Debug)]
pub struct InputSource {
    map: RawMap,
}

impl InputSource {
    pub fn events(&self) -> impl Iterator<Item = InputEvent> + '_ {
        MappedIter { map: &self.map }
    }
}

impl From<CharMap> for InputSource {
    fn from(cm: CharMap) -> Self {
        let mut result = InputSource {
            map: HashMap::with_capacity(cm.0.capacity()),
        };

        for (c, v) in cm.0 {
            let new_key = match c {
                '\n' => TEvent::Key(KeyEvent::from(Key::Enter)), // crossterm doesn't \n -> Enter
                '\t' => TEvent::Key(KeyEvent::from(Key::Tab)),   // crossterm doesn't \t -> Tab
                _ => TEvent::Key(KeyEvent::from(Key::Char(c))),
            };

            result.map.insert(new_key, v);
        }

        // meta-key defaults
        result.map.insert(TEvent::Key(KeyEvent::from(Key::Esc)), Event::Cancel);
        result
            .map
            .insert(TEvent::Key(KeyEvent::from(Key::Left)), Event::ArrowLeft);
        result
            .map
            .insert(TEvent::Key(KeyEvent::from(Key::Right)), Event::ArrowRight);
        result.map.insert(TEvent::Key(KeyEvent::from(Key::Up)), Event::ArrowUp);
        result
            .map
            .insert(TEvent::Key(KeyEvent::from(Key::Down)), Event::ArrowDown);
        result
            .map
            .insert(TEvent::Key(KeyEvent::from(Key::Delete)), Event::Delete);
        result
            .map
            .insert(TEvent::Key(KeyEvent::from(Key::Backspace)), Event::Backspace);
        result.map.insert(
            TEvent::Key(KeyEvent::new(Key::Char('h'), KeyModifiers::CONTROL)),
            Event::Left(MoveMeta::Alternative),
        );
        result.map.insert(
            TEvent::Key(KeyEvent::new(Key::Char('j'), KeyModifiers::CONTROL)),
            Event::Right(MoveMeta::Alternative),
        );
        result.map.insert(
            TEvent::Key(KeyEvent::new(Key::Char('k'), KeyModifiers::CONTROL)),
            Event::Up(MoveMeta::Alternative),
        );
        result.map.insert(
            TEvent::Key(KeyEvent::new(Key::Char('l'), KeyModifiers::CONTROL)),
            Event::Down(MoveMeta::Alternative),
        );
        result.map.insert(
            TEvent::Key(KeyEvent::from(Key::BackTab)),
            Event::SelectObject(Which::Next, true),
        );

        result
    }
}
