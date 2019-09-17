extern crate termion;

use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
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

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::InputHandler, "input_handler", &[])
        .with(
            systems::ActionHandler,
            "action_handler",
            &["input_handler"],
        )
        .with(
            systems::ClearScreen,
            "clear_screen",
            &["action_handler"],
        )
        .with(
            systems::SpriteRenderer,
            "sprite_renderer",
            &["clear_screen"],
        )
        .with(
            systems::BorderRenderer,
            "border_renderer",
            &["clear_screen"],
        )
        .with(
            systems::CmdLineRenderer,
            "cmdline_renderer",
            &["sprite_renderer", "border_renderer"],
        )
        .build();
    dispatcher.setup(&mut world);

    if args.len() > 1 {
        let sprite = components::Sprite::from_file(Path::new(&args[1])).unwrap();

        world
            .create_entity()
            .with(components::Dimension::for_sprite(&sprite).unwrap())
            .with(components::Position::from_xy(10, 10))
            .with(components::Selection) // pre-selected
            .with(components::Color(
                termion::color::AnsiValue::rgb(0, 5, 0).fg_string(),
            ))
            .with(components::Border)
            .with(sprite)
            .build();
    }

    world
        .create_entity()
        .with(components::Position::from_xy(2, 2))
        .with(components::Dimension::for_screen(&screen_size))
        .with(components::Border)
        .build();

    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    write!(stdout, "{}", termion::clear::All,).unwrap();

    dispatcher.dispatch(&world);
    world.maintain();

    world
        .fetch_mut::<resources::SyncTerm>()
        .flush_into(&mut stdout)
        .unwrap();
    stdout.flush().unwrap();

    for c in stdin().events() {
        // handle input
        world.fetch_mut::<resources::State>().push_event(c.unwrap());

        dispatcher.dispatch(&world);

        if world.fetch_mut::<resources::State>().mode() == resources::Mode::Quitting {
            break;
        }

        // TODO: only if dirty check fails
        world.maintain();
        dispatcher.dispatch(&world);

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
