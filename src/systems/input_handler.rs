use crate::common::{Action, Error};
use crate::components::{Direction, Selection, Translation};
use crate::resources::{CmdLine, ColorMode, ColorPalette, Mode, State};
use specs::{Read, ReadStorage, System, Write};
use termion::event::{Event, Key};

pub struct InputHandler;

impl InputHandler {
    fn objmode_event(event: Event, state: &mut State, selection: &ReadStorage<Selection>) {
        let ts = termion::terminal_size().unwrap();

        let action = match event {
            Event::Key(Key::Esc) => Action::ReverseMode,
            Event::Key(Key::Char(':')) => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                Action::SetMode(Mode::Command)
            }
            Event::Key(Key::Char('e')) => {
                if selection.count() == 1 {
                    state.push_action(Action::ClearError); // clean errors when to edit mode
                    Action::SetMode(Mode::Edit)
                } else {
                    state.set_error(Error::execution("One object must be selected"));
                    Action::None
                }
            }
            Event::Key(Key::Char('\t')) => Action::SelectNext(false),

            Event::Key(Key::Delete) | Event::Key(Key::Backspace) | Event::Key(Key::Char('d')) => {
                Action::Delete
            }

            Event::Key(Key::Char('u')) => Action::Undo,
            Event::Key(Key::Char('U')) => Action::Redo,

            Event::Key(Key::Char('i')) => Action::Translate(Translation::Relative(0, 0, -1)),
            Event::Key(Key::Char('m')) => Action::Translate(Translation::Relative(0, 0, 1)),

            Event::Key(Key::Char('F')) => Action::SetMode(Mode::Color(ColorMode::Fg)),
            Event::Key(Key::Char('B')) => Action::SetMode(Mode::Color(ColorMode::Bg)),
            Event::Key(Key::Char('f')) => Action::ApplyColor(ColorMode::Fg),
            Event::Key(Key::Char('b')) => Action::ApplyColor(ColorMode::Bg),

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

    fn palette_event(event: Event, state: &mut State, cm: ColorMode, palette: &ColorPalette) {
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

    fn edit_event(event: Event, state: &mut State) {
        match event {
            Event::Key(Key::Char(':')) => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                state.push_action(Action::SetMode(Mode::Command));
            }
            Event::Key(Key::Esc) => {
                use std::io::Write;
                writeln!(std::io::stderr(), "Esc pressed in: {:?}", state.mode()).unwrap();

                state.push_action(Action::ReverseMode)
            }
            _ => {}
        };
    }
}

impl<'a> System<'a> for InputHandler {
    type SystemData = (
        Write<'a, State>,
        Write<'a, CmdLine>,
        Read<'a, ColorPalette>,
        ReadStorage<'a, Selection>,
    );

    fn run(&mut self, (mut state, mut cmdline, palette, selection): Self::SystemData) {
        while let Some(event) = state.pop_event() {
            match state.mode() {
                Mode::Command => Self::cmdline_event(event, &mut state, &mut cmdline),
                Mode::Object => Self::objmode_event(event, &mut state, &selection),
                Mode::Color(cm) => Self::palette_event(event, &mut state, cm, &palette),
                Mode::Edit => Self::edit_event(event, &mut state),
                Mode::Quitting(_) => {}
            }
        }
    }
}
