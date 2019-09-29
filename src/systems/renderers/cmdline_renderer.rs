use crate::resources::{CmdLine, Mode, State, SyncTerm, ColorPalette};
use specs::System;
use std::io::Write;

pub struct CmdLineRenderer;

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
            write!(
                out,
                "{}{}{}{}",
                crate::common::goto(1, h),
                termion::color::Bg(termion::color::Red),
                error,
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
            return;
        }

        match state.mode() {
            Mode::Quitting(_) => return,
            Mode::Command => {
                write!(out, "{}:{}", crate::common::goto(1, h), cmdline.cmd()).unwrap()
            }
            Mode::Immediate => write!(
                out,
                "{}{}{}--INSERT--{}",
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
            Mode::Color => write!(
                out,
                "{}{}{}--COLOR--\t{}{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                palette.line_str(),
                termion::style::Reset,
            )
            .unwrap(),
        }
    }
}
