use crate::common::{cwd_path, Action, Error, Loaded, Loader, Scene, SceneV1, SymbolStyle};
use crate::components::*;
use crate::resources::{ColorMode, Mode, State, PALETTE_H, PALETTE_OFFSET, PALETTE_W};
use libflate::gzip::Encoder;
use specs::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
use std::path::{Path, PathBuf};

pub struct ActionHandler;

const NEW_POSITION: Position = Position { x: 10, y: 10, z: 0 };

impl<'a> System<'a> for ActionHandler {
    type SystemData = (
        Write<'a, State>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selectable>,
        ReadStorage<'a, Selection>,
        WriteStorage<'a, Subselection>,
        WriteStorage<'a, Position2D>, // cursor position saved to sprite
        WriteStorage<'a, Dimension>,
        WriteStorage<'a, Sprite>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (mut state, e, mut p, sel, s, mut ss, mut pss, mut d, mut sp, u): Self::SystemData) {        
        while let Some(action) = state.pop_action() {
            let keep_history = action.keeps_history();

            let changed = match action {
                Action::None => false,
                Action::Undo => undo(&mut state, &e, &s, &sp, &u),
                Action::Redo => redo(&mut state, &e, &s, &sp, &u),
                Action::NewObject => new_sprite(&mut state, &e, &s, &u, None),
                Action::ClearError => state.clear_error(),
                Action::SetMode(mode) => set_mode(mode, &mut state, &e, &s, &ss, &p, &pss, &u),
                Action::ApplyColor(cm) => apply_color_to_selected(cm, &state, &e, &mut sp, &p, &s, &d, &ss, &pss, &u),
                Action::ApplySymbol(sym) => apply_symbol_to_selected(sym, &mut state, &e, &mut sp, &s, &mut p, &mut d, &ss, &pss, &u),
                Action::ApplyStyle(style) => apply_style_to_selected(style, &state, &e, &mut sp, &p, &d, &s, &ss, &pss, &u),
                Action::ReverseMode => reverse_mode(&e, &mut state, &s, &ss, &mut pss, &u),
                Action::Deselect => match state.mode() {
                    Mode::Edit => clear_subselection(&e, &ss, &u),
                    _ => deselect_obj(&e, &s, &u),
                }
                Action::Translate(t) => match state.mode() {
                    Mode::Edit => {
                        let sprite_bounds = selected_bounds(&s, &p, &d);
                        translate_subselection(t, &mut state, &ss, &mut pss, &mut d, sprite_bounds)
                    },
                    _ => translate_selected(t, &mut state, &mut p, &s, &mut d),
                }
                Action::SelectNext(keep) => match state.mode() {
                    Mode::Object => select_next_obj(&e, &sel, &s, &u, keep),
                    Mode::Edit => select_edit(&e, &state, &mut ss, &u),
                    _ => state.set_error(Error::execution("Unexpected mode on selection")),
                }
                Action::Delete => {
                    if state.mode() == Mode::Edit || state.mode() == Mode::Write {
                        clear_symbol_on_selected(&mut state, &e, &mut sp, &s, &mut p, &mut d, &ss, &pss, &u)
                    } else if let Err(err) = delete_selected(&e, &s) {
                        state.set_error(err)
                    } else {
                        true
                    }
                }
                Action::Write(path) => {
                    if let Err(err) = save_scene(&e, &mut state, &sp, &p, &s, &path) {
                        state.set_error(err)
                    } else {
                        state.saved(path)
                    }
                }
                Action::Read(path) => {
                    if let Err(err) = load_from_file(&e, &s, &sp, &u, &path) {
                        state.set_error(err)
                    } else {
                        true
                    }
                }
            };

            if keep_history && changed {
                state.dirty();
            }
        }
    }
}

fn reverse_mode(
    e: &Entities,
    state: &mut State,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    cur_pos: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = clear_subselection(e, ss, u);

    if state.reverse_mode() {
        for (entity, _) in (e, s).join() {
            if let Some(cp) = cur_pos.get_mut(entity) {
                *cp = state.cursor; // update last cursor position
            } else {
                u.insert(entity, state.cursor); // insert last cursor position
            }
        }

        changed = true;
    }

    changed
}

fn deselect_obj(e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) -> bool {
    let mut changed = false;
    for (entity, _) in (e, s).join() {
        u.remove::<Selection>(entity);
        changed = true;
    }

    changed
}

fn clear_subselection(e: &Entities, ss: &WriteStorage<Subselection>, u: &LazyUpdate) -> bool {
    if let Some((entity, _)) = (e, ss).join().next() {
        u.remove::<Position2D>(entity);
        u.remove::<Dimension>(entity);
        u.remove::<Subselection>(entity);

        true
    } else {
        false
    }
}

fn set_mode(
    mode: Mode,
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    p: &WriteStorage<Position>,
    cur_pos: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    if match mode {
        Mode::Edit | Mode::Write => match s.count() {
            1 => {
                state.clear_error();
                for (entity, pos, _) in (e, p, s).join() {
                    if let Some(cp) = cur_pos.get(entity) {
                        state.cursor = *cp;
                    } else {
                        state.cursor = pos.into();
                    }
                }
                true
            }
            0 => {
                state.clear_error();
                state.cursor = (&NEW_POSITION).into();
                new_sprite(state, e, s, u, None)
            }
            _ => state.set_error(Error::execution("Multiple objects selected")),
        },
        Mode::SelectColor(_, _) => {
            let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
            state.cursor = Position2D {
                x: PALETTE_OFFSET,
                y: i32::from(ts.1) - 14,
            };

            true
        }
        _ => true,
    } {
        if state.set_mode(mode) {
            clear_subselection(e, ss, u);
        }

        true
    } else {
        false
    }
}

fn select_edit(
    e: &Entities,
    state: &State,
    ss: &mut WriteStorage<Subselection>,
    u: &LazyUpdate,
) -> bool {
    let mut joined = (e, ss).join();

    let clear_edit = |entity| {
        u.remove::<Position2D>(entity);
        u.remove::<Dimension>(entity);
        u.remove::<Subselection>(entity);
    };

    let new_edit = || {
        let entity = e.create();
        let pos: Position2D = state.cursor.into();
        u.insert(entity, pos);
        u.insert(entity, Dimension::unit());
        u.insert(entity, Subselection::at(pos));
    };

    if let Some((entity, sel)) = joined.next() { // existing selection, finish it
        if sel.active {
            sel.active = false; // we're done selecting
        } else { // redo
            clear_edit(entity);
            new_edit();
        }
    } else { // initiating new selection/edit
        new_edit();
    }

    false
}

fn select_next_obj(
    e: &Entities,
    sel: &ReadStorage<Selectable>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    keep: bool,
) -> bool {
    let mut changed = false;
    let mut all: Vec<(Entity, bool)> = Vec::default();
    let mut start = 0usize;

    for (i, (entity, _)) in (e, sel).join().enumerate() {
        let is_selected = s.contains(entity);
        all.push((entity, is_selected));
        if is_selected {
            start = i
        }
    }

    // go through unselected items only, starting with last selection known
    let mut unselected_iter = all
        .iter()
        .cycle()
        .skip(start)
        .take(all.len())
        .filter(|(_, is_sel)| !is_sel);

    if !keep {
        deselect_obj(e, s, u);
    }

    if let Some(entity) = unselected_iter.next() {
        changed = true;
        u.insert(entity.0, Selection); // select next if possible
    } else if let Some(entity) = all.first() {
        changed = true;
        u.insert(entity.0, Selection); // select first if "redeselecting"
    }

    changed
}

fn delete_selected(e: &Entities, s: &ReadStorage<Selection>) -> Result<(), Error> {
    let mut deleted = 0usize;

    for (entity, _) in (e, s).join() {
        if e.delete(entity).is_err() {
            return Err(Error::execution("Error deleting entity"));
        }

        deleted += 1;
    }

    if deleted == 0 {
        return Err(Error::execution("No entity to delete"));
    }

    Ok(())
}

fn selected_bounds(
    s: &ReadStorage<Selection>,
    p: &WriteStorage<Position>,
    d: &WriteStorage<Dimension>,
) -> Option<Bounds> {
    if let Some((position, _, dim)) = (p, s, d).join().next() {
        Some(Bounds::Free(position.into(), *dim))
    } else {
        None
    }
}

fn translate_subselection(
    t: Translation,
    state: &mut State,
    ss: &WriteStorage<Subselection>,
    p: &mut WriteStorage<Position2D>,
    d: &mut WriteStorage<Dimension>,
    sprite_bounds: Option<Bounds>,
) -> bool {
    if let Some(bounds) = sprite_bounds {
        if state.cursor.apply(t, bounds) {
            // if we have subselection, resize it
            if let Some((pos, dim, sel)) = (p, d, ss).join().next() {
                if !sel.active {
                    return true; // nothing to do
                }

                let edit_box = sel.initial_pos.area(state.cursor);

                *pos = *edit_box.position();
                *dim = *edit_box.dimension();
            }

            true
        } else {
            false
        }
    } else {
        false
    }
}

fn translate_selected(
    t: Translation,
    state: &mut State,
    p: &mut WriteStorage<Position>,
    s: &ReadStorage<Selection>,
    d: &WriteStorage<Dimension>,
) -> bool {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    let screen_dim = Dimension::from_wh(ts.0, ts.1);
    let palette_pos = Position2D {
        x: PALETTE_OFFSET,
        y: i32::from(ts.1) - 14,
    };
    let palette_dim = Dimension::from_wh(PALETTE_W as u16, PALETTE_H as u16);
    let palette_bounds = Bounds::Binding(palette_pos, palette_dim);

    let mode = state.mode();

    match mode {
        Mode::Object | Mode::Write => {
            let mut changed = false;

            for (position, _, dim) in (p, s, d).join() {
                let sprite_bounds = Bounds::Free(position.into(), *dim);
                let screen_bounds = Bounds::Free(Position2D::default(), screen_dim - *dim);

                if match state.mode() {
                    Mode::Write => state.cursor.apply(t, sprite_bounds),
                    Mode::Object => position.apply(t, screen_bounds),
                    _ => false,
                } {
                    changed = true;
                }
            }

            changed
        }
        Mode::SelectColor(_, _) => state.cursor.apply(t, palette_bounds),
        _ => false,
    }
}

fn apply_color_to_selected(
    cm: ColorMode,
    state: &State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    s: &ReadStorage<Selection>,
    d: &WriteStorage<Dimension>,
    ss: &WriteStorage<Subselection>,
    p_ss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let color = state.color(cm);

    for (sprite, pos, _) in (sp, p, s).join() {
        if state.mode() == Mode::Edit {
            let sel_bounds = subselection(ss, p_ss, d).unwrap_or_else(|| Bounds::point(state.cursor));
            let pos2d: Position2D = pos.into();
            let rel_bounds = sel_bounds - pos2d;
            
            if sprite.apply_color(cm, color, rel_bounds) {
                changed = true;
            }
            clear_subselection(e, ss, u);
        } else if sprite.fill_color(cm, color) {
            changed = true;
        }
    }

    changed
}

fn clear_symbol_on_selected(
    state: &mut State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    p: &mut WriteStorage<Position>,
    d: &mut WriteStorage<Dimension>,
    ss: &WriteStorage<Subselection>,
    p_ss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let sel_bounds = subselection(ss, p_ss, d).unwrap_or_else(|| Bounds::point(state.cursor));

    for (sprite, pos, dim, _) in (sp, p, d, s).join() {
        let pos2d: Position2D = pos.into();
        let rel_bounds = sel_bounds - pos2d;

        let changes = sprite.clear_symbol(rel_bounds);

        match changes {
            Ok(None) => {
                clear_subselection(e, ss, u);
            } // no change, symbol was applied in bounds
            Ok(Some(bounds)) => {
                // changed pos or dim => apply new bounds
                *pos += *bounds.position();
                *dim = *bounds.dimension();

                changed = true;
                clear_subselection(e, ss, u);
            }
            Err(err) => {
                // if dim is funky?
                return state.set_error(err);
            }
        }
    }

    changed
}

fn subselection(
    ss: &WriteStorage<Subselection>,
    p_ss: &WriteStorage<Position2D>,
    d: &WriteStorage<Dimension>,
) -> Option<Bounds> {
    if let Some((pos, dim, _)) = (p_ss, d, ss).join().next() {
        Some(Bounds::Binding(*pos, *dim))
    } else {
        None
    }
}

fn apply_style_to_selected(
    style: SymbolStyle,
    state: &State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    d: &WriteStorage<Dimension>,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    p_ss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;

    for (sprite, pos, _) in (sp, p, s).join() {
        if state.mode() == Mode::Edit {
            let sel_bounds = subselection(ss, p_ss, d).unwrap_or_else(|| Bounds::point(state.cursor));
            let pos2d: Position2D = pos.into();
            let rel_bounds = sel_bounds - pos2d;

            if sprite.apply_style(style, rel_bounds) {
                changed = true;
            }
            clear_subselection(e, ss, u);
        } else if sprite.fill_style(style) {
            changed = true;
        }
    }

    changed
}

fn apply_symbol_to_selected(
    symbol: char,
    state: &mut State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    p: &mut WriteStorage<Position>,
    d: &mut WriteStorage<Dimension>,
    ss: &WriteStorage<Subselection>,
    p_ss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let bg = state.color(ColorMode::Bg);
    let fg = state.color(ColorMode::Fg);
    let sel_bounds = subselection(ss, p_ss, d).unwrap_or_else(|| Bounds::point(state.cursor));

    for (sprite, pos, dim, _) in (sp, p, d, s).join() {
        let pos2d: Position2D = pos.into();
        let rel_bounds = sel_bounds - pos2d;
        let changes = sprite.apply_symbol(symbol, bg, fg, rel_bounds);

        match changes {
            Ok(bounds) => {
                clear_subselection(e, ss, u);
                // changed pos or dim => apply new bounds
                *pos += *bounds.position();
                *dim = *bounds.dimension();

                changed = true;
            }
            Err(err) => {
                // if dim is funky?
                return state.set_error(err);
            }
        }
    }

    changed
}

fn new_sprite(
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    pos: Option<Position>,
) -> bool {
    deselect_obj(e, s, u);
    let entity = e.create();
    let sprite = Sprite::default();

    let dim = match Dimension::for_sprite(&sprite) {
        Ok(d) => d,
        Err(err) => return state.set_error(err.into()),
    };

    u.insert(entity, dim);
    u.insert(entity, pos.unwrap_or(NEW_POSITION));
    u.insert(entity, Selection);
    u.insert(entity, Selectable);
    u.insert(entity, Border);
    u.insert(entity, sprite);

    true
}

fn import_sprite(
    sprite: Sprite,
    e: &Entities,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    pos: Option<Position>,
    pre_select: bool,
) -> Result<(), Error> {
    deselect_obj(e, s, u);
    let entity = e.create();

    u.insert(entity, Dimension::for_sprite(&sprite)?);
    u.insert(entity, pos.unwrap_or(NEW_POSITION));
    if pre_select {
        u.insert(entity, Selection);
    }
    u.insert(entity, Selectable);
    u.insert(entity, Border);
    u.insert(entity, sprite);

    Ok(())
}

fn save_scene(
    e: &Entities,
    state: &mut State,
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    s: &ReadStorage<Selection>,
    new_path: &Option<String>,
) -> Result<(), Error> {
    let path = state.save_file(new_path)?;
    let scene = Scene::V1(SceneV1::from((e, sp, p, s)));
    let ronified = ron::ser::to_string(&scene)?;
    let raw_path = if Path::new(&path).extension() != Some(std::ffi::OsStr::new("rgz")) {
        Path::new(&path).with_extension("rgz")
    } else {
        PathBuf::from(path)
    };
    let abs_path = cwd_path(&raw_path)?;

    let file = std::fs::File::create(abs_path)?;
    let mut encoder = Encoder::new(file)?;

    use std::io::Write;
    encoder.write_all(ronified.as_ref())?;
    encoder.finish().into_result()?;
    Ok(())
}

fn load_from_file(
    e: &Entities,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    u: &LazyUpdate,
    path: &str,
) -> Result<(), Error> {
    match Loader::from_file(path)? {
        Loaded::Scene(scene) => apply_scene(scene, e, s, sp, u),
        Loaded::Sprite(sprite) => import_sprite(sprite, e, s, u, None, true),
    }
}

fn clear_scene(e: &Entities, sp: &WriteStorage<Sprite>) -> Result<(), Error> {
    for (entity, _) in (e, sp).join() {
        e.delete(entity)?;
    }

    Ok(())
}

fn apply_scene(
    scene: Scene,
    e: &Entities,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    u: &LazyUpdate,
) -> Result<(), Error> {
    clear_scene(e, sp)?;

    let current = scene.current();
    for obj in current.objects {
        import_sprite(obj.0, e, s, u, Some(obj.1), obj.2)?;
    }

    Ok(())
}

fn undo(
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    u: &LazyUpdate,
) -> bool {
    if let Some(scene) = state.undo() {
        match apply_scene(scene, &e, &s, &sp, &u) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        }
    } else {
        state.set_error(Error::execution("Nothing to undo"));
        false
    }
}

fn redo(
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    u: &LazyUpdate,
) -> bool {
    if let Some(scene) = state.redo() {
        match apply_scene(scene, &e, &s, &sp, &u) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        }
    } else {
        state.set_error(Error::execution("Nothing to redo"));
        false
    }
}
