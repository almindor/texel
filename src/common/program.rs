use legion::prelude::*;
use std::io::stdout;
use std::path::Path;

use crate::common::{fio, Action, Config, ConfigV2, Event, InputEvent};
use crate::os::{InputSource, Terminal};
use crate::resources::{CmdLine, ColorPalette, FrameBuffer, State, SymbolPalette};
use crate::systems::*;

pub fn run(args: Vec<String>) {
    let ts = Terminal::terminal_size();
    check_terminal_size(ts);

    let universe = Universe::new();
    let mut world = universe.create_world();
    let config_file = dirs::config_dir().unwrap().join("texel/config.ron");
    let config = match fio::from_config_file(&config_file) {
        Ok(val) => val.current(), // ensures we upgrade if there's a version change
        Err(_) => Config::default().current(),
    };

    let mut out = FrameBuffer::new(usize::from(ts.0), usize::from(ts.1));
    let mut state = State::default();
    let input_source = build_resources(&config, &mut world);

    // initial clear screen
    let mut terminal = Terminal::new(stdout());
    terminal.blank_to_black();

    // load files as needed
    load_from(args, &mut state);
    // run/render initial screen
    TexelSystems::run(&mut world, &mut state, &mut out);

    // flush buffers to terminal
    out.flush_into(terminal.endpoint()).unwrap();

    loop {
        let mapped = input_source.next_event(state.mode());
        // handle input
        dispatch_input_event(mapped, &mut state, &mut out, &mut terminal);
        TexelSystems::run(&mut world, &mut state, &mut out);
        // flush buffers to terminal
        out.flush_into(terminal.endpoint()).unwrap();

        if state.quitting() {
            break;
        }
    }
    // reset tty back with clear screen
    terminal.restore();

    // save config
    save_config(config, &config_file, &world);
}

fn load_from(args: Vec<String>, state: &mut State) -> bool {
    if args.len() > 1 {
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

        true
    } else {
        false
    }
}

fn build_resources(config: &ConfigV2, world: &mut World) -> InputSource {
    // prep resources
    world.resources.insert(CmdLine::default());
    world.resources.insert(config.color_palette.clone());
    world.resources.insert(config.symbol_palette.clone());

    InputSource::from(config.char_map.clone())
}

fn dispatch_input_event(event: InputEvent, state: &mut State, out: &mut FrameBuffer, terminal: &mut Terminal) {
    // ensure we re-blank on resizes
    if event.0 == Event::Resize {
        terminal.blank_to_black();
        out.resize();
    } else {
        // otherwise just push event into input handler's pipeline
        state.push_event(event);
    }
}

fn check_terminal_size(ts: (u16, u16)) {
    if ts.0 < 60 || ts.1 < 16 {
        eprintln!("Terminal size too small, minimum 60x16 is required");
        std::process::exit(1);
    }
}

fn save_config(mut v2: ConfigV2, config_file: &Path, world: &World) {
    use std::ops::Deref;

    let cp = world.resources.get::<ColorPalette>().unwrap();
    let sp = world.resources.get::<SymbolPalette>().unwrap();

    v2.color_palette = cp.deref().clone();
    v2.symbol_palette = sp.deref().clone();

    let config = Config::from(v2);

    config.to_config_file(config_file).unwrap();
}
