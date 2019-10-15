use std::env;

mod common;
mod components;
mod resources;
mod systems;

fn main() {
    common::run(env::args().collect());
}
