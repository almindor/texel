#[cfg(feature = "ion")]
mod input_termion;
#[cfg(feature = "ion")]
mod tty_termion;

#[cfg(feature = "ion")]
pub use input_termion::InputSource;
#[cfg(feature = "ion")]
pub use tty_termion::Terminal;

#[cfg(feature = "crossplatform")]
mod input_crossterm;
#[cfg(feature = "crossplatform")]
mod tty_crossterm;

#[cfg(feature = "crossplatform")]
pub use input_crossterm::InputSource;
#[cfg(feature = "crossplatform")]
pub use tty_crossterm::Terminal;
