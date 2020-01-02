#[cfg(feature = "ion")]
mod input_termion;
#[cfg(feature = "ion")]
mod tty_termion;

#[cfg(feature = "ion")]
pub use input_termion::InputSource;
#[cfg(feature = "ion")]
pub use tty_termion::Terminal;

#[cfg(not(feature = "ion"))]
mod input_crossterm;
#[cfg(not(feature = "ion"))]
mod tty_crossterm;

#[cfg(not(feature = "ion"))]
pub use input_crossterm::InputSource;
#[cfg(not(feature = "ion"))]
pub use tty_crossterm::Terminal;
