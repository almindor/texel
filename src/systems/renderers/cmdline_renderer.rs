use crate::resources::{CmdLine, ColorMode, ColorPalette, Mode, State, SyncTerm};
use crate::common::Error;
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

        match state.mode() {
            Mode::Quitting(_) => return,
            Mode::Command => Self::print_cmd(&mut out, cmdline.cmd(), h),
            Mode::Edit => write!(
                out,
                "{}{}{}--EDIT--{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                termion::style::Reset,
            )
            .unwrap(),
            Mode::Object => write!(
                out,
                "{}{}{}--SELECT--{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                termion::style::Reset,
            )
            .unwrap(),
            Mode::Color(cm) => write!(
                out,
                "{}{}{}--COLOR--\t{}{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                palette.line_str(cm),
                termion::style::Reset,
            )
            .unwrap(),
        }

        // color selection
        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let w = i32::from(ts.0);
        let h = i32::from(ts.1);
        let sc = (state.color(ColorMode::Bg), state.color(ColorMode::Fg));
        write!(
            out,
            "{}{}{}{}{}{}{}",
            crate::common::goto(w - 12, h),
            ColorPalette::u8_to_bg(sc.0),
            ColorPalette::u8_to_fg(sc.1),
            "▞",
            ColorPalette::default_fg(),
            ColorPalette::default_bg(),
            crate::common::goto(cmdline.cursor_pos(), h),
        )
        .unwrap();
    }
}
