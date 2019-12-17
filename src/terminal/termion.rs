
struct Termion;

impl Terminal for Termion {
    fn goto(x: u16, y: u16) -> impl std::fmt::Display {
        termion::cursor::Goto(u_x, u_y)
    }
}