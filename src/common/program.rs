use specs::prelude::*;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

use crate::common::{fio, Action, Config, ConfigV1, InputMap};
use crate::resources::{ColorPalette, State, SymbolPalette, SyncTerm, Terminal};
use crate::systems::*;

pub fn run(args: Vec<String>) {
    check_terminal_size();

    let mut world = World::new();
    let config_file = dirs::config_dir().unwrap().join("texel/config.ron");
    let input_map = load_input_map(&config_file, &mut world);

    let (mut updater, mut renderer) = build_dispatchers();
    // setup dispatchers with world
    updater.setup(&mut world);
    renderer.setup(&mut world);

    // initial clear screen
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    SyncTerm::blank_to_black(&mut stdout);

    // load files as needed
    load_from(args, &mut world, &mut updater);
    // draw initial set
    renderer.dispatch(&world);
    // flush buffers to terminal
    flush_terminal(&mut stdout, &world);

    for c in stdin().events() {
        // handle input
        let mapped = input_map.map_input(c.unwrap());
        world.fetch_mut::<State>().push_event(mapped);
        updater.dispatch(&world);
        // quit if needed
        if world.fetch_mut::<State>().quitting() {
            break;
        }
        // ensure we lazy update
        world.maintain();
        // render only after world is up to date
        renderer.dispatch(&world);
        // flush buffers to terminal
        flush_terminal(&mut stdout, &world);
    }
    // reset tty back with clear screen
    SyncTerm::restore_terminal(&mut stdout);

    // save config
    save_config(&config_file, &world);
}

fn load_from(args: Vec<String>, world: &mut World, updater: &mut Dispatcher) {
    if args.len() > 1 {
        {
            let mut state = world.fetch_mut::<State>();

            // single non-existing file -> make it
            if (&args[1..]).len() == 1 && !std::path::Path::new(&args[1]).exists() {
                let path = args.get(1).unwrap();
                state.saved(path.into()); // consider this file our save file
            } else {
                for path in &args[1..] {
                    state.push_action(Action::Read(String::from(path)));
                }

                if args.len() == 2 {
                    let path = args.get(1).unwrap();
                    state.saved(path.into()); // consider this file our save file
                }
            }
        }

        updater.dispatch(&world);
        world.maintain();
    }
}

fn load_input_map(config_file: &Path, world: &mut World) -> InputMap {
    let config = match fio::from_config_file(config_file) {
        Ok(val) => val.current(), // ensures we upgrade if there's a version change
        Err(_) => Config::default().current(),
    };

    let ts = SyncTerm::terminal_size();

    // prep resources
    world.insert(SyncTerm::new(usize::from(ts.0), usize::from(ts.1)));
    world.insert(State::default());
    world.insert(config.color_palette);
    world.insert(config.symbol_palette);

    InputMap::from(config.char_map)
}

fn check_terminal_size() {
    let ts = SyncTerm::terminal_size();
    if ts.0 < 60 || ts.1 < 16 {
        eprintln!("Terminal size too small, minimum 60x16 is required");
        std::process::exit(1);
    }
}

fn flush_terminal(stdout: &mut Terminal, world: &World) {
    world.fetch_mut::<SyncTerm>().flush_into(stdout).unwrap();
    stdout.flush().unwrap();
}

fn build_dispatchers<'a, 'b>() -> (Dispatcher<'a, 'b>, Dispatcher<'a, 'b>) {
    // create dispatchers
    let updater = DispatcherBuilder::new()
        .with(InputHandler, "input_handler", &[])
        .with(ActionHandler, "action_handler", &["input_handler"])
        .build();
    let renderer = DispatcherBuilder::new()
        .with(HistoryHandler, "history_handler", &[]) // needs to run after world.update
        .with(ClearScreen, "clear_screen", &[])
        .with(SpriteRenderer, "sprite_renderer", &["clear_screen"])
        .with(SubselectionRenderer, "subselection_renderer", &["sprite_renderer"])
        .with(
            CmdLineRenderer,
            "cmdline_renderer",
            &["clear_screen", "sprite_renderer"],
        )
        .build();

    (updater, renderer)
}

fn save_config(config_file: &Path, world: &World) {
    let cp = world.fetch::<ColorPalette>();
    let sp = world.fetch::<SymbolPalette>();
    let config = Config::V1(ConfigV1::from((&*cp, &*sp)));
    config.to_config_file(config_file).unwrap();
}
