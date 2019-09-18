use specs::{Component, VecStorage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color(pub String);

impl Default for Color {
    fn default() -> Self {
        Color(termion::color::Fg(termion::color::White).to_string())
    }
}

impl Component for Color {
    type Storage = VecStorage<Self>;
}
