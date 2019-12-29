use std::io::Write;
use crate::resources::SyncTerm;

pub type Terminal = termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>;

impl SyncTerm {
    pub fn terminal_size() -> (u16, u16) {
        termion::terminal_size().unwrap() // this needs to panic since we lose output otherwise
    }

    pub fn restore_terminal(stdout: &mut Terminal) {
        let color_reset = termion::color::Reset;
        write!(
            stdout,
            "{}{}{}{}",
            termion::clear::All,
            color_reset.fg_str(),
            color_reset.bg_str(),
            termion::cursor::Goto(1, 1)
        )
        .unwrap();
        stdout.flush().unwrap();
    }

    pub fn goto(x: i32, y: i32) -> impl std::fmt::Display {
        // ensure we don't try to go to < 1 on any axis
        let o_x = std::cmp::max(1, x);
        let o_y = std::cmp::max(1, y);
        // TODO: figure out better way to handle this
        let u_x = o_x as u16;
        let u_y = o_y as u16;

        termion::cursor::Goto(u_x, u_y)
    }
}