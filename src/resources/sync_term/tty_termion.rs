use std::io::{Stdout, Write};
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use texel_types::SymbolStyle;

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

    pub fn reset_sequence() -> impl std::fmt::Display {
        termion::style::Reset
    }

    pub fn style_sequence<'a>(style: SymbolStyle) -> &'a dyn std::fmt::Display {
        match style {
            SymbolStyle::Bold => &termion::style::Bold,
            SymbolStyle::Italic => &termion::style::Italic,
            SymbolStyle::Underline => &termion::style::Underline,
        }
    }

    pub fn rgb_u8(r: u8, g: u8, b: u8) -> u8 {
        termion::color::AnsiValue::rgb(r, g, b).0
    }

    pub fn grayscale_u8(shade: u8) -> u8 {
        termion::color::AnsiValue::grayscale(shade).0
    }

    pub fn bg_color_sequence(color: u8) -> impl std::fmt::Display {
        termion::color::AnsiValue(color).bg_string()
    }

    pub fn fg_color_sequence(color: u8) -> impl std::fmt::Display {
        termion::color::AnsiValue(color).fg_string()
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
