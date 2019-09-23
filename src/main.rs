extern crate termion;

use std::env;
use std::io::{stdin, stdout, Write};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

mod common;
mod components;
mod resources;
mod systems;

use specs::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let screen_size = termion::terminal_size().unwrap();

    let mut world = World::new();
    world.insert(resources::SyncTerm::new(screen_size.0, screen_size.1));
    world.insert(resources::State::default());

    let mut updater = DispatcherBuilder::new()
        .with(systems::InputHandler, "input_handler", &[])
        .with(systems::ActionHandler, "action_handler", &["input_handler"])
        .build();

    let mut renderer = DispatcherBuilder::new()
        .with(systems::ClearScreen, "clear_screen", &[])
        .with(
            systems::SpriteRenderer,
            "sprite_renderer",
            &["clear_screen"],
        )
        .with(
            systems::CmdLineRenderer,
            "cmdline_renderer",
            &["sprite_renderer"],
        )
        .build();

    updater.setup(&mut world);
    renderer.setup(&mut world);

    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    write!(stdout, "{}", termion::clear::All,).unwrap();

    if args.len() > 1 {
        match resources::Loader::from_files(&args[1..]) {
            Ok(sprites) => {
                let mut state = world.fetch_mut::<resources::State>();

                for sprite in sprites {
                    state.push_action(common::Action::Import(sprite));
                }
            }
            Err(err) => world.fetch_mut::<resources::State>().set_error(Some(err)),
        }

        updater.dispatch(&world);
        world.maintain();
    }

    renderer.dispatch(&world);

    world
        .fetch_mut::<resources::SyncTerm>()
        .flush_into(&mut stdout)
        .unwrap();
    stdout.flush().unwrap();

    for c in stdin().events() {
        // handle input
        world.fetch_mut::<resources::State>().push_event(c.unwrap());
        updater.dispatch(&world);

        if world.fetch_mut::<resources::State>().quitting() {
            break;
        }

        world.maintain();
        renderer.dispatch(&world);

        world
            .fetch_mut::<resources::SyncTerm>()
            .flush_into(&mut stdout)
            .unwrap();
        stdout.flush().unwrap();
    }

    let color_reset = termion::color::Reset;
    write!(
        stdout,
        "{}{}{}{}",
        termion::clear::All,
        color_reset.fg_str(),
        color_reset.bg_str(),
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
    stdout.flush().unwrap();
}
