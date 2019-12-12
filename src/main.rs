mod common;
mod components;
mod resources;
mod systems;
mod exporters;

fn main() {
    common::run(std::env::args().collect());
}
