use crate::common::{CharMap, Event, InputEvent, Mode, ModesCharMap, MoveMeta};
use crossterm::event::{read, Event as TEvent, KeyCode as Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
use std::collections::HashMap;
use texel_types::{ColorMode, Position2D, Which};

type RawMap = HashMap<TEvent, Event>;

#[derive(Debug)]
pub struct InputSource {
    mode_maps: Vec<RawMap>,
}

impl InputSource {
    pub fn next_event(&self, mode: Mode) -> InputEvent {
        let map = self
            .mode_maps
            .get(mode.index())
            .unwrap_or_else(|| panic!("Mode map not found"));

        match read() {
            Err(err) => panic!(err),
            Ok(raw_event) => self.map_input(raw_event, map),
        }
    }

    fn map_input(&self, raw_event: TEvent, map: &RawMap) -> InputEvent {
        let mapped = map.get(&raw_event).copied().unwrap_or_else(|| match raw_event {
            TEvent::Resize(_, _) => Event::Resize,
            TEvent::Mouse(MouseEvent::Down(MouseButton::Left, x, y, km)) => {
                let pos = Position2D::from_xy(x.into(), y.into());
                let sticky = km == KeyModifiers::SHIFT; // never seems to get through

                Event::MouseDown(pos, sticky)
            }
            TEvent::Mouse(MouseEvent::Drag(MouseButton::Left, x, y, _)) => {
                let pos = Position2D::from_xy(x.into(), y.into());

                Event::MouseDrag(pos)
            }
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
    result.insert(TEvent::Key(KeyEvent::from(Key::Esc)), Event::Cancel);
    result.insert(TEvent::Key(KeyEvent::from(Key::Left)), Event::ArrowLeft);
    result.insert(TEvent::Key(KeyEvent::from(Key::Right)), Event::ArrowRight);
    result.insert(TEvent::Key(KeyEvent::from(Key::Up)), Event::ArrowUp);
    result.insert(TEvent::Key(KeyEvent::from(Key::Down)), Event::ArrowDown);
    result.insert(TEvent::Key(KeyEvent::from(Key::Delete)), Event::Delete);
    result.insert(TEvent::Key(KeyEvent::from(Key::Backspace)), Event::Backspace);
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('h'), KeyModifiers::CONTROL)),
        Event::Left(MoveMeta::Alternative),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('j'), KeyModifiers::CONTROL)),
        Event::Right(MoveMeta::Alternative),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('k'), KeyModifiers::CONTROL)),
        Event::Up(MoveMeta::Alternative),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('l'), KeyModifiers::CONTROL)),
        Event::Down(MoveMeta::Alternative),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('z'), KeyModifiers::ALT)),
        Event::PickColor(ColorMode::Fg),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('x'), KeyModifiers::ALT)),
        Event::PickColor(ColorMode::Bg),
    );
    result.insert(
        TEvent::Key(KeyEvent::from(Key::BackTab)),
        Event::SelectObject(Which::Next, true),
    );
    result.insert(
        TEvent::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::CONTROL)),
        Event::SelectObject(Which::All, false),
    );

    result
}

fn char_map_to_raw_map(cm: &CharMap, raw_map: &mut RawMap) {
    for (c, v) in &cm.0 {
        let mut ke = KeyEvent::from(Key::Char(*c));
        // need to map SHIFT on chars
        if c.is_uppercase() {
            ke.modifiers |= KeyModifiers::SHIFT;
        }
        let new_key = match c {
            '\n' => TEvent::Key(KeyEvent::from(Key::Enter)), // crossterm doesn't \n -> Enter
            '\t' => TEvent::Key(KeyEvent::from(Key::Tab)),   // crossterm doesn't \t -> Tab
            _ => TEvent::Key(ke),
        };

        raw_map.insert(new_key, *v);
    }
}
