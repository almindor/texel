use std::io::{Stdout, Write};
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;

type TermionTTY = termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>;

pub struct Terminal(TermionTTY);

impl Terminal {
    pub fn new(stdout: Stdout) -> Self {
        Terminal(MouseTerminal::from(stdout.into_raw_mode().unwrap()))
    }

    pub fn endpoint(&mut self) -> &mut dyn Write {
        &mut self.0
    }

    pub fn terminal_size() -> (u16, u16) {
        termion::terminal_size().unwrap() // this needs to panic since we lose output otherwise
    }

    pub fn restore(&mut self) {
        let color_reset = termion::color::Reset;
        write!(
            self.0,
            "{}{}{}{}",
            termion::clear::All,
            color_reset.fg_str(),
            color_reset.bg_str(),
            termion::cursor::Goto(1, 1)
        )
        .unwrap();

        self.0.flush().unwrap();
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

    pub fn blank_to_black(&mut self) {
        let ts = Self::terminal_size();
        let empty_line = " ".repeat(usize::from(ts.0));

        write!(self.0, "{}", termion::clear::All,).unwrap();

        for y in 1..=ts.1 {
            write!(
                self.0,
                "{}{}{}",
                Self::goto(1, i32::from(y)),
                termion::color::Bg(termion::color::AnsiValue(16)),
                empty_line,
            )
            .unwrap();
        }
    }
}
