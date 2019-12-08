mod common;
mod components;
mod resources;
mod systems;

fn main() {
    common::run(std::env::args().collect());
}
