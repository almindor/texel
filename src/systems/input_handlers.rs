use crate::common::{Action, Command};
use crate::components::{Direction, Translation};
use crate::resources::{CmdLine, Mode, State, SyncTerm};
use specs::{System, Write};
use termion::event::{Event, Key};

pub struct InputHandler;

impl InputHandler {
    fn objmode_event(&mut self, event: Event, state: &mut State, out: &SyncTerm) {   
        let action = match event {
            Event::Key(Key::Char(':')) => Action::SetMode(Mode::Command),
            Event::Key(Key::Char('h')) => Action::Translate(Translation::Relative(-1, 0)),
            Event::Key(Key::Char('j')) => Action::Translate(Translation::Relative(0, 1)),
            Event::Key(Key::Char('k')) => Action::Translate(Translation::Relative(0, -1)),
            Event::Key(Key::Char('l')) => Action::Translate(Translation::Relative(1, 0)),
            Event::Key(Key::Char('H')) => Action::Translate(Translation::ToEdge(Direction::Left(1))),
            Event::Key(Key::Char('J')) => Action::Translate(Translation::ToEdge(Direction::Top(1))),
            Event::Key(Key::Char('K')) => Action::Translate(Translation::ToEdge(Direction::Bottom(out.h))),
            Event::Key(Key::Char('L')) => Action::Translate(Translation::ToEdge(Direction::Right(out.w))),
            _ => Action::None,
        };

        state.push_action(action);
    }

    fn cmdline_event(&mut self, event: Event, state: &mut State, cmdline: &mut CmdLine) {
        match event {
            Event::Key(k) => match cmdline.input(k) {
                Ok(command) => match command {
                    Command::None => {}
                    Command::Quit => state.push_action(Action::SetMode(Mode::Quitting)),
                    Command::Cancel => state.push_action(Action::ReverseMode),
                    Command::Perform(action) => {
                        state.push_action(action);
                        state.push_action(Action::ReverseMode);
                    },
                    _ => {} // TODO
                },
                Err(error) => {
                    state.reverse_mode();
                    state.error = Some(error);
                }
            },
            _ => {}
        };
    }
}

impl<'a> System<'a> for InputHandler {
    type SystemData = (
        Write<'a, SyncTerm>,
        Write<'a, State>,
        Write<'a, CmdLine>,
    );

    fn run(&mut self, (out, mut state, mut cmdline): Self::SystemData) {
        while let Some(event) = state.pop_event() {
            match state.mode() {
                Mode::Command => self.cmdline_event(event, &mut state, &mut cmdline),
                Mode::Object => self.objmode_event(event, &mut state, &out),
                Mode::Immediate => {}
                Mode::Quitting => {}
            }
        }
    }
}
