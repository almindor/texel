use crate::resources::{CmdLine, Mode, State, SyncTerm};
use specs::{System};
use std::io::Write;

pub struct CmdLineRenderer;

impl<'a> System<'a> for CmdLineRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        specs::Read<'a, State>,
        specs::Read<'a, CmdLine>,
    );

    fn run(&mut self, (mut out, state, cmdline): Self::SystemData) {
        let h = i32::from(out.h);

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
        }
    }
}
