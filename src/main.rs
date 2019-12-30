mod common;
mod components;
mod exporters;
mod os;
mod resources;
mod systems;

fn main() {
    common::run(std::env::args().collect());
}
