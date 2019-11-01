use crate::common::Error;
use crate::resources::{CmdLine, ColorMode, ColorPalette, Mode, State, SymbolPalette, SyncTerm};
use specs::System;
use std::io::Write;

pub struct CmdLineRenderer;

pub const PALETTE_OFFSET: i32 = 24;

impl<'a> System<'a> for CmdLineRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        specs::Read<'a, State>,
        specs::Read<'a, CmdLine>,
        specs::Read<'a, ColorPalette>,
        specs::Read<'a, SymbolPalette>,
    );

    fn run(&mut self, (mut out, state, cmdline, color_palette, symbol_palette): Self::SystemData) {
        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let w = i32::from(ts.0);
        let h = i32::from(ts.1);

        print_status_line(&mut out, &state, w, h);

        if let Some(error) = state.error() {
            print_error(&mut out, error, h);
            return;
        }

        let mode = state.mode();
        match mode {
            Mode::Quitting(_) => return,
            Mode::Command => print_cmd(&mut out, cmdline.cmd(), h),
            Mode::Object => print_mode(&mut out, mode, w, h),
            Mode::Edit => print_edit(&mut out, &state, &symbol_palette, h),
            Mode::Color(cm) => print_color(&mut out, &color_palette, cm, w, h),
            Mode::SelectSymbol(i) | Mode::SelectColor(i) => print_palette(&mut out, mode, i, w, h),
        }
    }
}

fn print_error(out: &mut SyncTerm, error: &Error, h: i32) {
    write!(
        out,
        "{}{}{}{}",
        crate::common::goto(1, h),
        termion::color::Bg(termion::color::Red),
        error,
        termion::color::Bg(termion::color::Reset),
    )
    .unwrap();
}

fn print_cmd(out: &mut SyncTerm, cmd: &str, h: i32) {
    write!(out, "{}:{}", crate::common::goto(1, h), cmd).unwrap();
}

fn print_status_line(out: &mut SyncTerm, state: &State, w: i32, h: i32) {
    // color selection
    let sc = (state.color(ColorMode::Bg), state.color(ColorMode::Fg));

    write!(
        out,
        "{}{}{}{}{}{}",
        crate::common::goto(w - 12, h),
        ColorPalette::u8_to_bg(sc.0),
        ColorPalette::u8_to_fg(sc.1),
        "▞",
        ColorPalette::default_fg(),
        ColorPalette::default_bg(),
    )
    .unwrap();
}

fn print_mode(out: &mut SyncTerm, mode: Mode, w: i32, h: i32) {
    write!(
        out,
        "{}{}{}--{}--{}{}",
        crate::common::goto(1, h),
        termion::style::Bold,
        termion::color::Fg(termion::color::White),
        mode.to_str(),
        termion::style::Reset,
        crate::common::goto(w, h),
    )
    .unwrap();
}

fn print_edit(out: &mut SyncTerm, state: &State, palette: &SymbolPalette, h: i32) {
    write!(
        out,
        "{}{}{}--EDIT--\t{}{}{}",
        crate::common::goto(1, h),
        termion::style::Bold,
        termion::color::Fg(termion::color::White),
        palette.line_str(),
        termion::style::Reset,
        crate::common::goto(state.cursor.x, state.cursor.y),
    )
    .unwrap();
}

fn print_color(out: &mut SyncTerm, palette: &ColorPalette, cm: ColorMode, w: i32, h: i32) {
    write!(
        out,
        "{}{}{}--COLOR--{}{}{}{}",
        crate::common::goto(1, h),
        termion::style::Bold,
        termion::color::Fg(termion::color::White),
        crate::common::goto(PALETTE_OFFSET, h),
        palette.line_str(cm),
        termion::style::Reset,
        crate::common::goto(w, h),
    )
    .unwrap();
}

fn print_palette(out: &mut SyncTerm, mode: Mode, index: usize, w: i32, h: i32) {
    let i = crate::common::index_from_one(index);

    write!(
        out,
        "{}{}{}--{}--{}\t{}{:x?}{}",
        crate::common::goto(1, h),
        termion::style::Bold,
        termion::color::Fg(termion::color::White),
        mode.to_str(),
        termion::style::Reset,
        crate::common::goto(PALETTE_OFFSET + i - 1, h),
        i,
        crate::common::goto(w, h),
    )
    .unwrap();
}
