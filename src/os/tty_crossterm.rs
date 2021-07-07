use crossterm::terminal::size as crossterm_size;
use crossterm::ExecutableCommand;
use std::io::{Stdout, Write};
use texel_types::SymbolStyle;

pub struct Terminal(Stdout);

impl Terminal {
    pub fn new(mut stdout: Stdout) -> Self {
        crossterm::terminal::enable_raw_mode().unwrap();

        stdout.execute(crossterm::event::EnableMouseCapture).unwrap();

        Terminal(stdout)
    }

    pub fn endpoint(&mut self) -> &mut dyn Write {
        &mut self.0
    }

    pub fn terminal_size() -> (u16, u16) {
        crossterm_size().unwrap() // this needs to panic since we lose output otherwise
    }

    pub fn restore(&mut self) {
        let clear_cmd = crossterm::terminal::Clear(crossterm::terminal::ClearType::All).to_string();

        self.0.execute(crossterm::event::DisableMouseCapture).unwrap();

        write!(
            self.0,
            "{}{}{}",
            crossterm::style::ResetColor,
            &clear_cmd,
            crossterm::cursor::MoveTo(0, 0),
        )
        .unwrap();

        crossterm::terminal::disable_raw_mode().unwrap();

        self.0.flush().unwrap();
    }

    pub fn goto(x: i32, y: i32) -> impl std::fmt::Display {
        // ensure we don't try to go to < 0 on any axis
        let o_x = std::cmp::max(0, x);
        let o_y = std::cmp::max(0, y);
        // TODO: figure out better way to handle this
        let u_x = o_x as u16;
        let u_y = o_y as u16;

        crossterm::cursor::MoveTo(u_x, u_y)
    }

    pub fn reset_sequence() -> impl std::fmt::Display {
        crossterm::style::ResetColor
    }

    pub fn style_sequence(style: SymbolStyle) -> &'static dyn std::fmt::Display {
        use crossterm::style::{Attribute, SetAttribute};

        match style {
            SymbolStyle::Bold => &SetAttribute(Attribute::Bold),
            SymbolStyle::Italic => &SetAttribute(Attribute::Italic),
            SymbolStyle::Underline => &SetAttribute(Attribute::Underlined),
        }
    }

    pub fn rgb_u8(r: u8, g: u8, b: u8) -> u8 {
        16 + 36 * r + 6 * g + b
    }

    pub fn grayscale_u8(shade: u8) -> u8 {
        0xE8 + shade
    }

    pub fn bg_color_sequence(color: u8) -> impl std::fmt::Display {
        let localized = crossterm::style::Color::AnsiValue(color);

        crossterm::style::SetBackgroundColor(localized)
    }

    pub fn fg_color_sequence(color: u8) -> impl std::fmt::Display {
        let localized = crossterm::style::Color::AnsiValue(color);

        crossterm::style::SetForegroundColor(localized)
    }

    pub fn blank_to_black(&mut self) {
        let ts = Self::terminal_size();
        let empty_line = " ".repeat(usize::from(ts.0));

        write!(self.0, "{}", crossterm::style::ResetColor).unwrap();

        for y in 0..ts.1 {
            write!(
                self.0,
                "{}{}{}",
                Self::goto(0, i32::from(y)),
                Self::bg_color_sequence(16),
                empty_line,
            )
            .unwrap();
        }
    }
}
