use crate::common::Action;
use crate::components::{Direction, Translation};
use crate::resources::{CmdLine, ColorMode, ColorPalette, SymbolPalette, Mode, State};
use specs::{Read, System, Write};
use termion::event::{Event, Key};

pub struct InputHandler;

impl InputHandler {
    fn objmode_event(event: Event, state: &mut State) {
        let ts = termion::terminal_size().unwrap();

        let action = match event {
            Event::Key(Key::Esc) => Action::ReverseMode,
            Event::Key(Key::Char(':')) => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                Action::SetMode(Mode::Command)
            }
            Event::Key(Key::Char('e')) => Action::SetMode(Mode::Edit),
            Event::Key(Key::Char('\t')) => Action::SelectNext(false),

            Event::Key(Key::Delete) | Event::Key(Key::Backspace) | Event::Key(Key::Char('d')) => {
                Action::Delete
            }

            Event::Key(Key::Char('u')) => Action::Undo,
            Event::Key(Key::Char('U')) => Action::Redo,

            Event::Key(Key::Char('g')) => Action::Translate(Translation::Relative(0, 0, -1)),
            Event::Key(Key::Char('y')) => Action::Translate(Translation::Relative(0, 0, 1)),

            Event::Key(Key::Char('F')) => Action::SetMode(Mode::Color(ColorMode::Fg)),
            Event::Key(Key::Char('B')) => Action::SetMode(Mode::Color(ColorMode::Bg)),
            Event::Key(Key::Char('q')) => Action::ApplyColor(ColorMode::Fg),
            Event::Key(Key::Char('w')) => Action::ApplyColor(ColorMode::Bg),

            Event::Key(Key::Char('h')) => Action::Translate(Translation::Relative(-1, 0, 0)),
            Event::Key(Key::Char('j')) => Action::Translate(Translation::Relative(0, 1, 0)),
            Event::Key(Key::Char('k')) => Action::Translate(Translation::Relative(0, -1, 0)),
            Event::Key(Key::Char('l')) => Action::Translate(Translation::Relative(1, 0, 0)),
            Event::Key(Key::Char('H')) => {
                Action::Translate(Translation::ToEdge(Direction::Left(1)))
            }
            Event::Key(Key::Char('J')) => Action::Translate(Translation::ToEdge(Direction::Top(1))),
            Event::Key(Key::Char('K')) => {
                Action::Translate(Translation::ToEdge(Direction::Bottom(ts.1)))
            }
            Event::Key(Key::Char('L')) => {
                Action::Translate(Translation::ToEdge(Direction::Right(ts.0)))
            }
            Event::Unsupported(raw) => {
                // shift + tab
                if raw.len() == 3 && raw[0] == 27 && raw[1] == 91 && raw[2] == 90 {
                    Action::SelectNext(true)
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        };

        state.push_action(action);
    }

    fn cmdline_event(event: Event, state: &mut State, cmdline: &mut CmdLine) {
        match event {
            Event::Key(k) => match cmdline.input(k) {
                Ok(action) => {
                    if action.is_some() && !action.is_reverse_mode() {
                        state.push_action(Action::ReverseMode);
                    }
                    state.push_action(action);
                }
                Err(err) => {
                    state.set_error(err);
                    state.push_action(Action::ReverseMode);
                }
            },
            _ => {}
        };
    }

    fn color_event(event: Event, state: &mut State, cm: ColorMode, palette: &ColorPalette) {
        match event {
            Event::Key(Key::Char(':')) => {
                state.push_action(Action::ReverseMode);
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                state.push_action(Action::SetMode(Mode::Command));
            }
            Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                state.push_action(Action::ReverseMode)
            }
            Event::Key(Key::Char(c)) => {
                if let Some(index) = c.to_digit(16) {
                    state.set_color(palette.color(index as usize), cm);
                    state.push_action(Action::ReverseMode);
                }
            }
            _ => {}
        };
    }

    fn edit_event(event: Event, state: &mut State, palette: &SymbolPalette) {
        let action = match event {
            Event::Key(Key::Char(':')) => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                Action::SetMode(Mode::Command)
            }
            Event::Key(Key::Esc) => Action::ReverseMode,

            Event::Key(Key::Delete) | Event::Key(Key::Backspace) | Event::Key(Key::Char('d')) => {
                Action::Delete
            }

            Event::Key(Key::Char('u')) => Action::Undo,
            Event::Key(Key::Char('U')) => Action::Redo,

            Event::Key(Key::Char('g')) => Action::Translate(Translation::Relative(0, 0, -1)),
            Event::Key(Key::Char('y')) => Action::Translate(Translation::Relative(0, 0, 1)),

            Event::Key(Key::Char('F')) => Action::SetMode(Mode::Color(ColorMode::Fg)),
            Event::Key(Key::Char('B')) => Action::SetMode(Mode::Color(ColorMode::Bg)),
            Event::Key(Key::Char('q')) => Action::ApplyColor(ColorMode::Fg),
            Event::Key(Key::Char('w')) => Action::ApplyColor(ColorMode::Bg),

            Event::Key(Key::Char('h')) => Action::Translate(Translation::Relative(-1, 0, 0)),
            Event::Key(Key::Char('j')) => Action::Translate(Translation::Relative(0, 1, 0)),
            Event::Key(Key::Char('k')) => Action::Translate(Translation::Relative(0, -1, 0)),
            Event::Key(Key::Char('l')) => Action::Translate(Translation::Relative(1, 0, 0)),

            Event::Key(Key::Char(c)) => {
                if let Some(index) = c.to_digit(16) {
                    Action::ApplySymbol(palette.symbol(index as usize))
                } else {
                    Action::None
                }
            }
            // Event::Key(Key::Char('H')) => {
            //     Action::Translate(Translation::ToEdge(Direction::Left(1)))
            // }
            // Event::Key(Key::Char('J')) => Action::Translate(Translation::ToEdge(Direction::Top(1))),
            // Event::Key(Key::Char('K')) => {
            //     Action::Translate(Translation::ToEdge(Direction::Bottom(ts.1)))
            // }
            // Event::Key(Key::Char('L')) => {
            //     Action::Translate(Translation::ToEdge(Direction::Right(ts.0)))
            // }
            _ => Action::None,
        };

        state.push_action(action);
    }
}

impl<'a> System<'a> for InputHandler {
    type SystemData = (Write<'a, State>, Write<'a, CmdLine>,
        Read<'a, ColorPalette>,
        Read<'a, SymbolPalette>);

    fn run(&mut self, (mut state, mut cmdline, color_palette, symbol_palette): Self::SystemData) {
        while let Some(event) = state.pop_event() {
            match state.mode() {
                Mode::Command => Self::cmdline_event(event, &mut state, &mut cmdline),
                Mode::Object => Self::objmode_event(event, &mut state),
                Mode::Color(cm) => Self::color_event(event, &mut state, cm, &color_palette),
                Mode::Edit => Self::edit_event(event, &mut state, &symbol_palette),
                Mode::Quitting(_) => {}
            }
        }
    }
}
