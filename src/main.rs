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

    let mut world = World::new();
    world.insert(resources::SyncTerm::new());
    world.insert(resources::State::default());
    world.insert(resources::ColorPalette::default());

    let mut updater = DispatcherBuilder::new()
        .with(systems::InputHandler, "input_handler", &[])
        .with(systems::ActionHandler, "action_handler", &["input_handler"])
        .build();

    let mut renderer = DispatcherBuilder::new()
        .with(systems::HistoryHandler, "history_handler", &[]) // a bit wonky but works fine
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
        {
            let mut state = world.fetch_mut::<resources::State>();

            for path in &args[1..] {
                state.push_action(common::Action::Load(String::from(path)));
            }
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
