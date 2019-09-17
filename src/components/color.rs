use specs::{Component, VecStorage};

pub struct Color(pub String);

impl Default for Color {
    fn default() -> Self {
        Color(termion::color::Fg(termion::color::White).to_string())
    }
}

impl Component for Color {
    type Storage = VecStorage<Self>;
}
