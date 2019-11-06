use specs::prelude::*;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

use crate::common::{Action, Config, ConfigV1, Error, InputMap, Loader};
use crate::resources::{ColorPalette, State, SymbolPalette, SyncTerm};
use crate::systems::*;

type Terminal = termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>;

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
    write!(stdout, "{}", termion::clear::All).unwrap();
    // load files as needed
    load_from(args, &mut world, &mut updater, &mut renderer);
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
    restore_terminal(&mut stdout);

    // save config
    save_config(&config_file, &world);
}

fn load_from(args: Vec<String>, world: &mut World, updater: &mut Dispatcher, renderer: &mut Dispatcher) {
    if args.len() > 1 {
        {
            let mut state = world.fetch_mut::<State>();

            // single non-existing file -> make it
            if (&args[1..]).len() == 1 && !std::path::Path::new(&args[1]).exists() {
                let path = args.get(1).unwrap();
                state.saved(Some(path.into())); // consider this file our save file
            } else {
                for path in &args[1..] {
                    state.push_action(Action::Read(String::from(path)));
                }
            }
        }

        updater.dispatch(&world);
        world.maintain();
        renderer.dispatch(&world); // due to history handler

        // must set saved state after history handler is done
        let mut state = world.fetch_mut::<State>();
        if args.len() == 2 {
            if let Some(path) = args.get(1) {
                state.saved(Some(String::from(path))); // store saved state with filename
            } else {
                state.set_error(Error::execution("Unable to determine source file"));
            }
        } else {
            // loaded multiple, store save state but with no file
            state.saved(None);
        }
    } else {
        // render first time
        renderer.dispatch(&world);
    }
}

fn load_input_map(config_file: &Path, world: &mut World) -> InputMap {
    let config = match Loader::from_config_file(config_file) {
        Ok(val) => val.current(), // ensures we upgrade if there's a version change
        Err(_) => Config::default().current(),
    };

    // prep resources
    world.insert(SyncTerm::new());
    world.insert(State::default());
    world.insert(config.color_palette);
    world.insert(config.symbol_palette);

    InputMap::from(config.char_map)
}

fn check_terminal_size() {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    if ts.0 < 60 || ts.1 < 16 {
        writeln!(std::io::stderr(), "Terminal size too small, minimum 60x16 is required").unwrap();
        std::process::exit(1);
    }
}

fn flush_terminal(stdout: &mut Terminal, world: &World) {
    world.fetch_mut::<SyncTerm>().flush_into(stdout).unwrap();
    stdout.flush().unwrap();
}

fn restore_terminal(stdout: &mut Terminal) {
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
        .with(ColorPaletteRenderer, "color_palette_renderer", &["sprite_renderer"])
        .with(
            CmdLineRenderer,
            "cmdline_renderer",
            &["clear_screen", "color_palette_renderer", "sprite_renderer"],
        )
        .build();

    (updater, renderer)
}

fn save_config(config_file: &Path, world: &World) {
    let cp = world.fetch::<ColorPalette>();
    let sp = world.fetch::<SymbolPalette>();
    let config = Config::V1(ConfigV1::from((&*cp, &*sp)));
    Loader::to_config_file(config, config_file).unwrap();
}
