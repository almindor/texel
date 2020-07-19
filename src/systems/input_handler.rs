use crate::common::{Action, Event, InputEvent, Mode, MoveMeta, SelectMode};
use crate::components::{Direction, Translation};
use crate::resources::{CmdLine, ColorPalette, State, SymbolPalette};
use legion::prelude::*;
use texel_types::ColorMode;

pub fn handle_input(world: &mut World, state: &mut State) {
    let cmdline = &mut world.resources.get_mut::<CmdLine>().unwrap();
    let symbol_palette = &mut world.resources.get_mut::<SymbolPalette>().unwrap();
    let color_palette = &mut world.resources.get_mut::<ColorPalette>().unwrap();

    while let Some(event) = state.pop_event() {
        match state.mode() {
            Mode::Command => cmdline_event(event, state, cmdline),
            Mode::Object(SelectMode::Object) => objmode_event(event, state),
            Mode::Object(SelectMode::Region) => objmode_region_event(event, state),
            Mode::Color(cm) => color_event(event, state, cm, color_palette),
            Mode::SelectSymbol(index) => symbol_select_event(event, state, index, symbol_palette),
            Mode::SelectColor(index, _) => color_select_event(event, state, index, color_palette),
            Mode::Edit => edit_event(event, state, &symbol_palette),
            Mode::Write => write_event(event, state),
            Mode::Help(_) => help_event(event, state),
            Mode::Quitting(_) => {}
        }
    }
}

fn objmode_event(event: InputEvent, state: &mut State) {
    let action = match event.0 {
        Event::Mode(mode) => Action::SetMode(mode),
        Event::SelectObject(which, sticky) => Action::SelectObject(which, sticky),
        Event::SelectFrame(which) => Action::SelectFrame(which),
        Event::SelectRegion => Action::SetMode(Mode::Object(SelectMode::Region)),
        Event::SelectPalette(index) => {
            if index < 10 {
                Action::Bookmark(index, false)
            } else {
                Action::None
            }
        }
        Event::EditPalette(index) => {
            if index < 10 {
                Action::Bookmark(index, true)
            } else {
                Action::None
            }
        }

        Event::Cancel => Action::Cancel,
        Event::Delete | Event::Backspace => Action::Delete,

        Event::DeleteFrame => Action::DeleteFrame,
        Event::NewFrame => Action::NewFrame,

        Event::Undo => Action::Undo,
        Event::Redo => Action::Redo,

        Event::Clipboard(op) => Action::Clipboard(op),
        Event::ToggleMetadata => Action::ToggleMetadata,
        Event::NewObject => Action::NewObject,
        Event::Duplicate(count) => Action::Duplicate(count),
        Event::Deselect => Action::Deselect,

        Event::Above => Action::Translate(Translation::Relative(0, 0, -1)),
        Event::Below => Action::Translate(Translation::Relative(0, 0, 1)),

        Event::ApplyColor(cm) => Action::ApplyColor(cm),
        Event::ApplyStyle(style) => Action::ApplyStyle(style),

        Event::Left(MoveMeta::Relative) | Event::ArrowLeft => Action::Translate(Translation::Relative(-1, 0, 0)),
        Event::Up(MoveMeta::Relative) | Event::ArrowUp => Action::Translate(Translation::Relative(0, -1, 0)),
        Event::Down(MoveMeta::Relative) | Event::ArrowDown => Action::Translate(Translation::Relative(0, 1, 0)),
        Event::Right(MoveMeta::Relative) | Event::ArrowRight => Action::Translate(Translation::Relative(1, 0, 0)),

        Event::Left(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Left)),
        Event::Up(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Top)),
        Event::Down(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Bottom)),
        Event::Right(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Right)),

        _ => Action::None,
    };

    state.push_action(action);
}

fn objmode_region_event(event: InputEvent, state: &mut State) {
    let action = match event.0 {
        Event::Mode(mode) => Action::SetMode(mode),
        Event::SelectRegion => Action::SelectRegion,

        Event::Cancel => Action::Cancel,
        Event::Confirm => Action::ApplyRegion,
        Event::Delete | Event::Backspace => {
            // TODO: this is currently broken because actionhandler does the pop
            // select and delete
            state.push_action(Action::ApplyRegion);
            Action::Delete
        }

        Event::Undo => Action::Undo,
        Event::Redo => Action::Redo,

        Event::Clipboard(op) => Action::Clipboard(op),
        Event::ToggleMetadata => Action::ToggleMetadata,
        Event::Deselect => Action::Deselect,

        Event::Left(MoveMeta::Relative) | Event::ArrowLeft => Action::Translate(Translation::Relative(-1, 0, 0)),
        Event::Up(MoveMeta::Relative) | Event::ArrowUp => Action::Translate(Translation::Relative(0, -1, 0)),
        Event::Down(MoveMeta::Relative) | Event::ArrowDown => Action::Translate(Translation::Relative(0, 1, 0)),
        Event::Right(MoveMeta::Relative) | Event::ArrowRight => Action::Translate(Translation::Relative(1, 0, 0)),

        Event::Left(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Left)),
        Event::Up(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Top)),
        Event::Down(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Bottom)),
        Event::Right(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Right)),

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
        Event::Mode(Mode::Command) => {
            state.push_action(Action::ReverseMode);
            state.push_action(Action::SetMode(Mode::Command));
        }
        Event::EditPalette(index) => state.push_action(Action::SetMode(Mode::SelectColor(index, cm))),
        Event::SelectPalette(index) => {
            state.set_color(palette.color(index), cm);
            state.push_action(Action::ReverseMode);
        }
        Event::Cancel => state.push_action(Action::Cancel),
        _ => {}
    };
}

fn write_event(event: InputEvent, state: &mut State) {
    let action = match event.0 {
        Event::Cancel => Action::Cancel,

        Event::ArrowLeft => Action::Translate(Translation::Relative(-1, 0, 0)),
        Event::ArrowUp => Action::Translate(Translation::Relative(0, -1, 0)),
        Event::ArrowDown => Action::Translate(Translation::Relative(0, 1, 0)),
        Event::ArrowRight => Action::Translate(Translation::Relative(1, 0, 0)),

        Event::Confirm => {
            state.push_action(Action::Translate(Translation::ToEdge(Direction::Left)));
            Action::Translate(Translation::Relative(0, 1, 0))
        }

        Event::Delete | Event::Backspace => {
            state.push_action(Action::Translate(Translation::Relative(-1, 0, 0))); // TODO: let action handler keep bounds and move up
            Action::Delete
        }

        Event::Deselect => Action::Deselect,

        _ => {
            if let Some(c) = event.1 {
                state.push_action(Action::ApplySymbol(c));
                Action::Translate(Translation::Relative(1, 0, 0))
            } else {
                Action::None
            }
        }
    };

    state.push_action(action);
}

fn edit_event(event: InputEvent, state: &mut State, palette: &SymbolPalette) {
    let action = match event.0 {
        Event::Mode(mode) => Action::SetMode(mode),
        Event::EditPalette(index) => Action::SetMode(Mode::SelectSymbol(index)),
        Event::SelectPalette(index) => Action::ApplySymbol(palette.symbol(index)),
        Event::Clipboard(op) => Action::Clipboard(op),
        Event::ToggleMetadata => Action::ToggleMetadata,

        Event::Cancel => Action::Cancel,
        Event::Delete | Event::Backspace => Action::Delete,
        Event::DeleteFrame => Action::DeleteFrame,
        Event::NewFrame => Action::NewFrame,
        Event::SelectRegion => Action::SelectRegion,
        Event::SelectFrame(which) => Action::SelectFrame(which),

        Event::Undo => Action::Undo,
        Event::Redo => Action::Redo,

        Event::Deselect => Action::Deselect,

        Event::Above => Action::Translate(Translation::Relative(0, 0, -1)),
        Event::Below => Action::Translate(Translation::Relative(0, 0, 1)),

        Event::PickColor(cm) => Action::PickColor(cm),
        Event::ApplyColor(cm) => Action::ApplyColor(cm),
        Event::ApplyStyle(style) => Action::ApplyStyle(style),

        Event::Left(MoveMeta::Relative) | Event::ArrowLeft => Action::Translate(Translation::Relative(-1, 0, 0)),
        Event::Up(MoveMeta::Relative) | Event::ArrowUp => Action::Translate(Translation::Relative(0, -1, 0)),
        Event::Down(MoveMeta::Relative) | Event::ArrowDown => Action::Translate(Translation::Relative(0, 1, 0)),
        Event::Right(MoveMeta::Relative) | Event::ArrowRight => Action::Translate(Translation::Relative(1, 0, 0)),

        Event::Left(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Left)),
        Event::Up(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Top)),
        Event::Down(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Bottom)),
        Event::Right(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Right)),

        _ => Action::None,
    };

    state.push_action(action);
}

fn symbol_select_event(event: InputEvent, state: &mut State, index: usize, palette: &mut SymbolPalette) {
    match event.0 {
        Event::Cancel => state.push_action(Action::Cancel),
        _ => {
            if let Some(c) = event.1 {
                match palette.set_symbol(index, c) {
                    Ok(_) => state.push_action(Action::ReverseMode),
                    Err(err) => {
                        state.set_error(err);
                    }
                };
            }
        }
    };
}

fn color_select_event(event: InputEvent, state: &mut State, index: usize, palette: &mut ColorPalette) {
    let action = match event.0 {
        Event::Cancel => Action::Cancel,
        Event::Confirm => {
            if let Err(err) = palette.set_color(index, ColorPalette::pos_to_color(state.cursor)) {
                state.set_error(err);
            }
            Action::ReverseMode
        }

        Event::Left(MoveMeta::Relative) | Event::ArrowLeft => Action::Translate(Translation::Relative(-1, 0, 0)),
        Event::Up(MoveMeta::Relative) | Event::ArrowUp => Action::Translate(Translation::Relative(0, -1, 0)),
        Event::Down(MoveMeta::Relative) | Event::ArrowDown => Action::Translate(Translation::Relative(0, 1, 0)),
        Event::Right(MoveMeta::Relative) | Event::ArrowRight => Action::Translate(Translation::Relative(1, 0, 0)),

        Event::Left(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Left)),
        Event::Up(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Top)),
        Event::Down(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Bottom)),
        Event::Right(MoveMeta::ToEdge) => Action::Translate(Translation::ToEdge(Direction::Right)),

        _ => Action::None,
    };

    state.push_action(action);
}

fn help_event(event: InputEvent, state: &mut State) {
    let action = match event.0 {
        Event::Mode(Mode::Command) => Action::SetMode(Mode::Command),
        _ => Action::ReverseMode,
    };

    state.push_action(action);
}
