use crate::common::Error;
use crate::resources::{CmdLine, ColorMode, ColorPalette, Mode, State, SyncTerm};
use specs::System;
use std::io::Write;

pub struct CmdLineRenderer;

impl CmdLineRenderer {
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

    fn print_status_line(out: &mut SyncTerm, sc: (u8, u8), cp: i32, w: i32, h: i32) {
        write!(
            out,
            "{}{}{}{}{}{}{}",
            crate::common::goto(w - 12, h),
            ColorPalette::u8_to_bg(sc.0),
            ColorPalette::u8_to_fg(sc.1),
            "▞",
            ColorPalette::default_fg(),
            ColorPalette::default_bg(),
            crate::common::goto(cp, h), // : + one after
        )
        .unwrap();
    }

    fn print_mode(out: &mut SyncTerm, mode: Mode, h: i32) {
        write!(
            out,
            "{}{}{}--{}--{}",
            crate::common::goto(1, h),
            termion::style::Bold,
            termion::color::Fg(termion::color::White),
            mode.to_str(),
            termion::style::Reset,
        )
        .unwrap();
    }

    fn print_palette(out: &mut SyncTerm, palette: &ColorPalette, cm: ColorMode, h: i32) {
        write!(
            out,
            "{}{}{}--COLOR--\t{}{}",
            crate::common::goto(1, h),
            termion::style::Bold,
            termion::color::Fg(termion::color::White),
            palette.line_str(cm),
            termion::style::Reset,
        )
        .unwrap();
    }
}

impl<'a> System<'a> for CmdLineRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        specs::Read<'a, State>,
        specs::Read<'a, CmdLine>,
        specs::Read<'a, ColorPalette>,
    );

    fn run(&mut self, (mut out, state, cmdline, palette): Self::SystemData) {
        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let h = i32::from(ts.1);

        if let Some(error) = state.error() {
            Self::print_error(&mut out, error, h);
            return;
        }

        let mode = state.mode();
        match mode {
            Mode::Quitting(_) => return,
            Mode::Command => Self::print_cmd(&mut out, cmdline.cmd(), h),
            Mode::Edit | Mode::Object => Self::print_mode(&mut out, mode, h),
            Mode::Color(cm) => Self::print_palette(&mut out, &palette, cm, h),
        }

        // color selection
        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let w = i32::from(ts.0);
        let h = i32::from(ts.1);
        let sc = (state.color(ColorMode::Bg), state.color(ColorMode::Fg));

        Self::print_status_line(&mut out, sc, cmdline.cursor_pos() + 2, w, h);
    }
}
