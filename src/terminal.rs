
mod termion;

pub use termion::Termion;

pub trait Terminal {
    fn goto(x: u16, y: u16) -> impl std::fmt::Display;
}
