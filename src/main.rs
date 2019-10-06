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
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    if ts.0 < 80 || ts.1 < 15 {
        writeln!(
            std::io::stderr(),
            "Terminal size too small, minimum 80x15 is required"
        )
        .unwrap();
        std::process::exit(1);
    }

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
        .with(systems::HistoryHandler, "history_handler", &[]) // needs to run after world.update
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
                state.push_action(common::Action::Read(String::from(path)));
            }
        }

        updater.dispatch(&world);
        world.maintain();
        renderer.dispatch(&world); // due to history handler

        // must set saved state after history handler is done
        let mut state = world.fetch_mut::<resources::State>();
        if args.len() == 2 {
            if let Some(path) = args.get(1) {
                state.saved(Some(String::from(path))); // store saved state with filename
            } else {
                state.set_error(common::Error::execution("Unable to determine source file"));
            }
        } else {
            // loaded multiple, store save state but with no file
            state.saved(None);
        }
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
