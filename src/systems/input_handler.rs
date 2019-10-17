use crate::common::{Action, Event, InputEvent};
use crate::components::{Direction, Translation};
use crate::resources::{CmdLine, ColorMode, ColorPalette, Mode, State, SymbolPalette};
use specs::{Read, System, Write};

pub struct InputHandler;

impl InputHandler {
    fn objmode_event(event: InputEvent, state: &mut State) {
        let ts = termion::terminal_size().unwrap();

        let action = match event.0 {
            Event::ModeCmd => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                Action::SetMode(Mode::Command)
            }
            Event::ModeEdit => Action::SetMode(Mode::Edit),
            Event::ModeColorFG => Action::SetMode(Mode::Color(ColorMode::Fg)),
            Event::ModeColorBG => Action::SetMode(Mode::Color(ColorMode::Bg)),

            Event::Next => Action::SelectNext(false),
            Event::NextWith => Action::SelectNext(true),

            Event::Cancel => Action::ReverseMode,
            Event::Delete | Event::Backspace => Action::Delete,

            Event::Undo => Action::Undo,
            Event::Redo => Action::Redo,

            Event::NewObject => Action::NewObject,

            Event::Above => Action::Translate(Translation::Relative(0, 0, -1)),
            Event::Below => Action::Translate(Translation::Relative(0, 0, 1)),

            Event::ApplyColorFG => Action::ApplyColor(ColorMode::Fg),
            Event::ApplyColorBG => Action::ApplyColor(ColorMode::Bg),

            Event::Left => Action::Translate(Translation::Relative(-1, 0, 0)),
            Event::Up => Action::Translate(Translation::Relative(0, -1, 0)),
            Event::Down => Action::Translate(Translation::Relative(0, 1, 0)),
            Event::Right => Action::Translate(Translation::Relative(1, 0, 0)),

            Event::LeftEdge => Action::Translate(Translation::ToEdge(Direction::Left(1))),
            Event::UpEdge => Action::Translate(Translation::ToEdge(Direction::Top(1))),
            Event::DownEdge => Action::Translate(Translation::ToEdge(Direction::Bottom(ts.1))),
            Event::RightEdge => Action::Translate(Translation::ToEdge(Direction::Right(ts.0))),

            _ => Action::None,
        };

        state.push_action(action);
    }

    fn cmdline_event(event: InputEvent, state: &mut State, cmdline: &mut CmdLine) {
        match cmdline.input(event) {
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
        };
    }

    fn color_event(event: InputEvent, state: &mut State, cm: ColorMode, palette: &ColorPalette) {
        match event.0 {
            Event::ModeCmd => {
                state.push_action(Action::ReverseMode);
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                state.push_action(Action::SetMode(Mode::Command));
            }
            Event::Cancel => state.push_action(Action::ReverseMode),
            _ => {}
        };

        if let Some(index) = event.1.and_then(|c| c.to_digit(16)) {
            state.set_color(palette.color(index as usize), cm);
            state.push_action(Action::ReverseMode);
        }
    }

    fn edit_event(event: InputEvent, state: &mut State, palette: &SymbolPalette) {
        let action = match event.0 {
            Event::ModeCmd => {
                state.push_action(Action::ClearError); // clean errors when going back to cmdline
                Action::SetMode(Mode::Command)
            }
            Event::ModeSymbol(index) => Action::SetMode(Mode::Symbol(index)),
            Event::Cancel => Action::ReverseMode,
            Event::Delete | Event::Backspace => Action::Delete,

            Event::Undo => Action::Undo,
            Event::Redo => Action::Redo,

            Event::Above => Action::Translate(Translation::Relative(0, 0, -1)),
            Event::Below => Action::Translate(Translation::Relative(0, 0, 1)),

            Event::ModeColorFG => Action::SetMode(Mode::Color(ColorMode::Fg)),
            Event::ModeColorBG => Action::SetMode(Mode::Color(ColorMode::Bg)),
            Event::ApplyColorFG => Action::ApplyColor(ColorMode::Fg),
            Event::ApplyColorBG => Action::ApplyColor(ColorMode::Bg),

            Event::Left => Action::Translate(Translation::Relative(-1, 0, 0)),
            Event::Up => Action::Translate(Translation::Relative(0, -1, 0)),

            Event::Down => Action::Translate(Translation::Relative(0, 1, 0)),
            Event::Right => Action::Translate(Translation::Relative(1, 0, 0)),

            // TODO: handle Edge movements
            _ => {
                if let Some(index) = event.1.and_then(|c| c.to_digit(16)) {
                    Action::ApplySymbol(palette.symbol(index as usize))
                } else {
                    Action::None
                }
            }
        };

        state.push_action(action);
    }

    fn symbol_event(event: InputEvent, state: &mut State, index: usize, palette: &mut SymbolPalette) {
        match event.0 {
            Event::Cancel => state.push_action(Action::ReverseMode),
            _ => {
                if let Some(c) = event.1 {
                    match palette.set_symbol(index, c) {
                        Ok(_) => state.push_action(Action::ReverseMode),
                        Err(err) => {
                            state.set_error(err);
                        },
                    };
                }
            }
        };
    }
}

impl<'a> System<'a> for InputHandler {
    type SystemData = (
        Write<'a, State>,
        Write<'a, CmdLine>,
        Write<'a, SymbolPalette>,
        Read<'a, ColorPalette>,
    );

    fn run(&mut self, (mut state, mut cmdline, mut symbol_palette, color_palette): Self::SystemData) {
        while let Some(event) = state.pop_event() {
            match state.mode() {
                Mode::Command => Self::cmdline_event(event, &mut state, &mut cmdline),
                Mode::Object => Self::objmode_event(event, &mut state),
                Mode::Color(cm) => Self::color_event(event, &mut state, cm, &color_palette),
                Mode::Symbol(index) => Self::symbol_event(event, &mut state, index, &mut symbol_palette),
                Mode::Edit => Self::edit_event(event, &mut state, &symbol_palette),
                Mode::Quitting(_) => {}
            }
        }
    }
}
