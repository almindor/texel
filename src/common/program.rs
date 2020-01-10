use specs::prelude::*;
use std::io::stdout;
use std::path::Path;

use crate::common::{fio, Action, Config, ConfigV1, Event, InputEvent};
use crate::os::{InputSource, Terminal};
use crate::resources::{ColorPalette, FrameBuffer, State, SymbolPalette};
use crate::systems::*;

pub fn run(args: Vec<String>) {
    check_terminal_size();

    let mut world = World::new();
    let config_file = dirs::config_dir().unwrap().join("texel/config.ron");
    let config = match fio::from_config_file(&config_file) {
        Ok(val) => val.current(), // ensures we upgrade if there's a version change
        Err(_) => Config::default().current(),
    };
    let input_source = build_resources(&config, &mut world);

    let mut renderer = build_dispatchers();
    let mut input_handler = InputHandler;
    let mut action_handler = ActionHandler;
    // setup dispatchers with world
    specs::System::setup(&mut input_handler, &mut world);
    specs::System::setup(&mut action_handler, &mut world);
    renderer.setup(&mut world);

    // initial clear screen
    let mut terminal = Terminal::new(stdout());
    terminal.blank_to_black();

    // load files as needed
    if load_from(args, &mut world) {
        action_handler.run_now(&world);
        world.maintain();
    }
    // draw initial set
    renderer.dispatch(&world);
    // flush buffers to terminal
    world
        .fetch_mut::<FrameBuffer>()
        .flush_into(terminal.endpoint())
        .unwrap();

    for mapped in input_source.events() {
        // handle input
        dispatch_input_event(&world, mapped, &mut terminal);
        input_handler.run_now(&world);
        // apply each action until done
        while world.fetch_mut::<State>().queued_actions() > 0 {
            action_handler.run_now(&world);
            world.maintain();
        }
        // quit if needed
        if world.fetch_mut::<State>().quitting() {
            break;
        }
        // ensure we lazy update
        // render only after world is up to date
        renderer.dispatch(&world);
        // flush buffers to terminal
        world
            .fetch_mut::<FrameBuffer>()
            .flush_into(terminal.endpoint())
            .unwrap();
    }
    // reset tty back with clear screen
    terminal.restore();

    // save config
    save_config(config, &config_file, &world);
}

fn load_from(args: Vec<String>, world: &mut World) -> bool {
    if args.len() > 1 {
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

        true
    } else {
        false
    }
}

fn build_resources(config: &ConfigV1, world: &mut World) -> InputSource {
    let ts = Terminal::terminal_size();

    // prep resources
    world.insert(FrameBuffer::new(usize::from(ts.0), usize::from(ts.1)));
    world.insert(State::default());
    world.insert(config.color_palette.clone());
    world.insert(config.symbol_palette.clone());

    InputSource::from(config.char_map.clone())
}

fn dispatch_input_event(world: &World, event: InputEvent, terminal: &mut Terminal) {
    // ensure we re-blank on resizes
    if event.0 == Event::Resize {
        terminal.blank_to_black();
        world.fetch_mut::<FrameBuffer>().resize();
    } else {
        // otherwise just push event into input handler's pipeline
        world.fetch_mut::<State>().push_event(event);
    }
}

fn check_terminal_size() {
    let ts = Terminal::terminal_size();
    if ts.0 < 60 || ts.1 < 16 {
        eprintln!("Terminal size too small, minimum 60x16 is required");
        std::process::exit(1);
    }
}

fn build_dispatchers<'a, 'b>() -> Dispatcher<'a, 'b> {
    // create renderer dispatcher
    let renderer = DispatcherBuilder::new()
        .with(HistoryHandler, "history_handler", &[])
        .with(ClearScreen, "clear_screen", &[])
        .with(SpriteRenderer, "sprite_renderer", &["clear_screen"])
        .with(SubselectionRenderer, "subselection_renderer", &["sprite_renderer"])
        .with(
            CmdLineRenderer,
            "cmdline_renderer",
            &["clear_screen", "sprite_renderer"],
        )
        .build();

    renderer
}

fn save_config(mut v1: ConfigV1, config_file: &Path, world: &World) {
    use std::ops::Deref;

    let cp = world.fetch::<ColorPalette>();
    let sp = world.fetch::<SymbolPalette>();

    v1.color_palette = cp.deref().clone();
    v1.symbol_palette = sp.deref().clone();

    let config = Config::from(v1);

    config.to_config_file(config_file).unwrap();
}
