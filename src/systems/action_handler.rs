use crate::common::{fio, Action, Clipboard, ClipboardOp, Error, Layout, Mode, OnQuit, Scene, SceneExt, SelectMode};
use crate::components::*;
use crate::os::Terminal;
use crate::resources::{State, PALETTE_H, PALETTE_OFFSET, PALETTE_W};
use fio::ExportFormat;
use specs::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
use texel_types::{ColorMode, SymbolStyle, Texels, Which};

pub struct ActionHandler;

const NEW_POSITION: Position = Position { x: 10, y: 10, z: 0 };

impl<'a> System<'a> for ActionHandler {
    type SystemData = (
        Write<'a, State>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selectable>,
        ReadStorage<'a, Selection>,
        ReadStorage<'a, Bookmark>,
        WriteStorage<'a, Subselection>,
        WriteStorage<'a, Position2D>, // cursor position saved to sprite, bookmark position
        WriteStorage<'a, Dimension>,
        WriteStorage<'a, Sprite>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (mut state, e, mut p, sel, s, b, mut ss, mut pss, mut d, mut sp, u): Self::SystemData) {
        while let Some(action) = state.pop_action() {
            let keep_history = action.keeps_history();

            let changed = match action {
                Action::None => false,
                Action::Undo => undo(&mut state, &e, &s, &sp, &b, &mut pss, &u),
                Action::Redo => redo(&mut state, &e, &s, &sp, &b, &mut pss, &u),
                Action::Clipboard(op) => clipboard(op, &mut state, &e, &mut sp, &s, &ss, &mut p, &pss, &mut d, &u),
                Action::NewObject => new_sprite(&state, &e, &s, &u, None),
                Action::Duplicate(count) => match duplicate_selected(count, &state, &e, &p, &sp, &s, &u) {
                    Ok(_) => true,
                    Err(err) => state.set_error(err),
                },
                Action::NewFrame => new_frame_on_selected(&mut state, &mut sp, &s),
                Action::DeleteFrame => delete_frame_on_selected(&mut state, &mut sp, &s),
                Action::Viewport(index, true) => set_bookmark(index, state.offset, &e, &b, &mut pss, &u),
                Action::Viewport(index, false) => jump_to_bookmark(index, &mut state, &b, &pss),
                Action::Cancel => {
                    if state.error().is_some() {
                        state.clear_error()
                    } else if !reverse_mode(&e, &mut state, &s, &ss, &p, &mut pss, &u) {
                        deselect_obj(&e, &s, &u)
                    } else {
                        false
                    }
                }
                Action::ClearError => state.clear_error(),
                Action::SetMode(mode) => set_mode(mode, &mut state, &e, &s, &ss, &sp, &p, &pss, &b, &u),
                Action::ApplyColor(cm) => apply_color_to_selected(cm, &state, &e, &mut sp, &p, &s, &d, &ss, &pss, &u),
                Action::ApplySymbol(sym) => {
                    apply_symbol_to_selected(sym, &mut state, &e, &mut sp, &s, &mut p, &mut d, &ss, &pss, &u)
                }
                Action::ApplyStyle(style) => {
                    apply_style_to_selected(style, &state, &e, &mut sp, &p, &d, &s, &ss, &pss, &u)
                }
                Action::ApplyRegion => {
                    apply_region(subselection(&ss, &pss, &d), &mut state, &e, &sel, &p, &d, &s, &ss, &u)
                }
                Action::ReverseMode => reverse_mode(&e, &mut state, &s, &ss, &p, &mut pss, &u),
                Action::Deselect => clear_subselection(&e, &ss, &u) || deselect_obj(&e, &s, &u),
                Action::Translate(t) => match state.mode() {
                    Mode::Edit => {
                        let sprite_bounds = selected_bounds(&s, &p, &d);
                        translate_subselection(t, &mut state, &mut ss, &mut pss, &mut d, sprite_bounds)
                    }
                    Mode::Object(SelectMode::Region) => {
                        let viewport_bounds = viewport_bounds(&state);
                        translate_subselection(t, &mut state, &mut ss, &mut pss, &mut d, viewport_bounds)
                    }
                    _ => translate_selected(t, &mut state, &mut p, &s, &d),
                },
                Action::Layout(layout) => apply_layout_to_selected(layout, &state, &mut p, &d, &s),
                Action::SelectFrame(which) => change_frame_on_selected(which, &mut state, &mut sp, &s),
                Action::SelectObject(which, sticky) => select_obj(which, &e, &sel, &s, &u, sticky),
                Action::SelectRegion => select_region(&mut state, &e, &mut ss, &sel, &p, &d, &s, &u),
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
                    if let Err(err) = save_scene(&mut state, &sp, &p, &b, &pss, &path) {
                        state.set_error(err)
                    } else if let Some(path) = path {
                        state.saved(path)
                    } else {
                        state.clear_changes()
                    }
                }
                Action::Read(path) => match load_from_file(&e, &mut state, &s, &sp, &b, &mut pss, &u, &path) {
                    Ok(changed) => changed, // we reset history in some cases here
                    Err(err) => state.set_error(err),
                },
                Action::Tutorial => match tutorial(&e, &mut state, &s, &sp, &b, &mut pss, &u) {
                    Ok(changed) => changed,
                    Err(err) => state.set_error(err),
                },
                Action::Export(format, path) => {
                    if let Err(err) = export_to_file(format, &path, &sp, &p, &b, &pss) {
                        state.set_error(err)
                    } else {
                        false
                    }
                }
                Action::ShowHelp(index) => {
                    state.set_mode(Mode::Help(index));
                    false
                }
                Action::ClearBlank => clear_blank_texels(&mut state, &mut sp, &s),
            };

            state.dirty = keep_history && changed;
        }
    }
}

fn reverse_mode(
    e: &Entities,
    state: &mut State,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    p: &WriteStorage<Position>,
    cur_pos: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    if state.reverse_mode() {
        for (entity, pos, _) in (e, p, s).join() {
            let pos2d: Position2D = pos.into();
            if let Some(cp) = cur_pos.get_mut(entity) {
                *cp = state.cursor - pos2d; // update to last cursor position
            } else {
                u.insert(entity, state.cursor - pos2d); // insert last cursor position
            }
        }

        true
    } else {
        clear_subselection(e, ss, u)
    }
}

fn deselect_obj(e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) -> bool {
    for (entity, _) in (e, s).join() {
        u.remove::<Selection>(entity);
    }

    false
}

fn clear_subselection(e: &Entities, ss: &WriteStorage<Subselection>, u: &LazyUpdate) -> bool {
    if let Some((entity, _)) = (e, ss).join().next() {
        u.remove::<Position2D>(entity);
        u.remove::<Dimension>(entity);
        u.remove::<Subselection>(entity);
    }

    false
}

fn set_mode(
    mode: Mode,
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    cur_pos: &WriteStorage<Position2D>,
    b: &ReadStorage<Bookmark>,
    u: &LazyUpdate,
) -> bool {
    if match mode {
        Mode::Quitting(OnQuit::Check) => {
            if state.unsaved_changes() {
                state.set_error(Error::execution("Unsaved changes, use q! to quit without saving"));
                false
            } else {
                true
            }
        }
        Mode::Quitting(OnQuit::Save) => {
            if state.unsaved_changes() {
                if let Err(err) = save_scene(state, sp, p, b, cur_pos, &None) {
                    state.set_error(err)
                } else {
                    true
                }
            } else {
                true
            }
        }
        Mode::Edit | Mode::Write => match s.count() {
            1 => {
                state.clear_error();
                if let Some((entity, pos, _)) = (e, p, s).join().next() {
                    let pos2d: Position2D = pos.into();

                    // if we're going from a non-cursory mode
                    if state.mode() != Mode::Edit && state.mode() != Mode::Write {
                        if let Some(cp) = cur_pos.get(entity) {
                            state.cursor = *cp + pos2d;
                        } else {
                            state.cursor = pos2d;
                        }
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
            let ts = Terminal::terminal_size();

            state.cursor = Position2D {
                x: PALETTE_OFFSET,
                y: i32::from(ts.1) - 14,
            };

            true
        }
        Mode::Object(SelectMode::Region) => {
            deselect_obj(e, s, u);
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

fn select_region(
    state: &mut State,
    e: &Entities,
    ss: &mut WriteStorage<Subselection>,
    sel: &ReadStorage<Selectable>,
    p: &WriteStorage<Position>,
    d: &WriteStorage<Dimension>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
) -> bool {
    match state.mode() {
        Mode::Edit => {
            mark_subselection(e, state, ss, u);
            false
        }
        Mode::Object(SelectMode::Region) => {
            apply_region(mark_subselection(e, state, ss, u), state, e, sel, p, d, s, ss, u)
        }
        _ => state.set_error(Error::execution("Region select in unexpected mode")),
    }
}

fn mark_subselection(
    e: &Entities,
    state: &State,
    ss: &mut WriteStorage<Subselection>,
    u: &LazyUpdate,
) -> Option<Bounds> {
    let mut joined = (e, ss).join();

    let clear_edit = |entity| {
        u.remove::<Position2D>(entity);
        u.remove::<Dimension>(entity);
        u.remove::<Subselection>(entity);
    };

    let new_edit = || {
        let entity = e.create();
        let pos = state.cursor;
        u.insert(entity, pos);
        u.insert(entity, Dimension::unit());
        u.insert(entity, Subselection::at(pos));
    };

    if let Some((entity, sel)) = joined.next() {
        // existing selection, finish it
        if sel.active {
            sel.active = false; // we're done selecting
            Some(sel.initial_pos.area(state.cursor))
        } else {
            // redo
            clear_edit(entity);
            new_edit();
            None
        }
    } else {
        // initiating new selection/edit
        new_edit();
        None
    }
}

fn apply_region(
    region: Option<Bounds>,
    state: &mut State,
    e: &Entities,
    sel: &ReadStorage<Selectable>,
    p: &WriteStorage<Position>,
    d: &WriteStorage<Dimension>,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    u: &LazyUpdate,
) -> bool {
    let area = match region {
        Some(bounds) => bounds,
        None => return false,
    };

    deselect_obj(e, s, u);

    for (entity, pos, dim, _) in (e, p, d, sel).join() {
        // any point inside region -> select
        let pos2d: Position2D = pos.into();
        if area.intersects(pos2d - state.offset, *dim) {
            u.insert(entity, Selection);
        }
    }

    clear_subselection(e, ss, u);
    state.reverse_mode();

    false
}

fn select_obj_relative(
    forward: bool, // TODO: support inverse selection
    e: &Entities,
    sel: &ReadStorage<Selectable>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    sticky: bool,
) -> bool {
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

    if !sticky {
        deselect_obj(e, s, u);
    }

    if let Some(entity) = unselected_iter.next() {
        u.insert(entity.0, Selection); // select next if possible
    } else if let Some(entity) = all.first() {
        u.insert(entity.0, Selection); // select first if "redeselecting"
    }

    false
}

fn select_obj_all(e: &Entities, sel: &ReadStorage<Selectable>, s: &ReadStorage<Selection>, u: &LazyUpdate) -> bool {
    for (entity, _) in (e, sel).join() {
        if !s.contains(entity) {
            u.insert(entity, Selection);
        }
    }

    false
}

fn select_obj_at(
    pos: Position2D,
    e: &Entities,
    sel: &ReadStorage<Selectable>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    sticky: bool,
) -> bool {
    // TODO
    false
}

fn select_obj(
    which: Which<Position2D>,
    e: &Entities,
    sel: &ReadStorage<Selectable>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    sticky: bool,
) -> bool {
    match which {
        Which::Next => select_obj_relative(true, e, sel, s, u, sticky),
        Which::Previous => select_obj_relative(false, e, sel, s, u, sticky),
        Which::All => select_obj_all(e, sel, s, u),
        Which::At(pos) => select_obj_at(pos, e, sel, s, u, sticky),
    }
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

fn viewport_bounds(state: &State) -> Option<Bounds> {
    let ts = Terminal::terminal_size();
    Some(Bounds::Free(state.offset, Dimension::from_wh(ts.0, ts.1)))
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
    ss: &mut WriteStorage<Subselection>,
    pss: &mut WriteStorage<Position2D>,
    d: &mut WriteStorage<Dimension>,
    area_bounds: Option<Bounds>,
) -> bool {
    if let Some(bounds) = area_bounds {
        if state.cursor.apply(t, bounds) {
            // if we have a subselection
            if let Some((ss_pos, sub_sel, dim)) = (pss, ss, d).join().next() {
                if sub_sel.active {
                    // adjusting subselection
                    let edit_box = sub_sel.initial_pos.area(state.cursor);
                    *ss_pos = *edit_box.position();
                    *dim = *edit_box.dimension();
                }
            }
        }
    }

    false
}

fn translate_selected(
    t: Translation,
    state: &mut State,
    p: &mut WriteStorage<Position>,
    s: &ReadStorage<Selection>,
    d: &WriteStorage<Dimension>,
) -> bool {
    let ts = Terminal::terminal_size();
    let screen_dim = Dimension::from_wh(ts.0, ts.1);
    let palette_pos = Position2D {
        x: PALETTE_OFFSET,
        y: i32::from(ts.1) - 14,
    };
    let palette_dim = Dimension::from_wh(PALETTE_W as u16, PALETTE_H as u16);
    let palette_bounds = Bounds::Binding(palette_pos, palette_dim);

    let mode = state.mode();

    match mode {
        Mode::Object(_) | Mode::Write => {
            let mut changed = false;

            // nothing selected, move viewport
            if s.count() == 0 {
                let screen_bounds = Bounds::Free(Position2D::default(), screen_dim);
                state.offset.apply(t, screen_bounds);
            }

            for (position, _, dim) in (p, s, d).join() {
                let sprite_bounds = Bounds::Free(position.into(), *dim);
                let screen_bounds = Bounds::Free(Position2D::default(), screen_dim - *dim);

                if match state.mode() {
                    Mode::Write => state.cursor.apply(t, sprite_bounds),
                    Mode::Object(_) => position.apply(t, screen_bounds),
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
    pss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let color = state.color(cm);

    for (sprite, pos, _) in (sp, p, s).join() {
        if state.mode() == Mode::Edit {
            let sel_bounds = subselection(ss, pss, d).unwrap_or_else(|| Bounds::point(state.cursor));
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
    pss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let sel_bounds = subselection(ss, pss, d).unwrap_or_else(|| Bounds::point(state.cursor));

    for (sprite, pos, dim, _) in (sp, p, d, s).join() {
        let pos2d: Position2D = pos.into();
        let rel_bounds = sel_bounds - pos2d;

        match sprite.clear_symbol(rel_bounds) {
            None => {
                changed = true;
                clear_subselection(e, ss, u);
            } // no change, symbol was applied in bounds
            Some(bounds) => {
                // changed pos or dim => apply new bounds
                *pos += *bounds.position();
                *dim = *bounds.dimension();

                changed = true;
                clear_subselection(e, ss, u);
            }
        }
    }

    changed
}

fn subselection(
    ss: &WriteStorage<Subselection>,
    pss: &WriteStorage<Position2D>,
    d: &WriteStorage<Dimension>,
) -> Option<Bounds> {
    if let Some((pos, dim, _)) = (pss, d, ss).join().next() {
        Some(Bounds::Binding(*pos, *dim))
    } else {
        None
    }
}

fn new_frame_on_selected(state: &mut State, sp: &mut WriteStorage<Sprite>, s: &ReadStorage<Selection>) -> bool {
    let mut changed = false;

    if s.count() == 0 {
        state.set_error(Error::execution("No objects selected"));
        return false;
    }

    for (sprite, _) in (sp, s).join() {
        sprite.new_frame();
        changed = true;
    }

    changed
}

fn delete_frame_on_selected(state: &mut State, sp: &mut WriteStorage<Sprite>, s: &ReadStorage<Selection>) -> bool {
    let mut changed = false;

    if s.count() == 0 {
        state.set_error(Error::execution("No objects selected"));
        return false;
    }

    for (sprite, _) in (sp, s).join() {
        if sprite.delete_frame() {
            changed = true;
        }
    }

    changed
}

fn set_bookmark(
    index: usize,
    location: Position2D,
    e: &Entities,
    b: &ReadStorage<Bookmark>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    if let Some((_, pos)) = (b, pb).join().find(|(bm, _)| bm.0 == index) {
        if location != *pos {
            *pos = location;
        }
    } else {
        let entity = e.create();
        u.insert(entity, Bookmark(index));
        u.insert(entity, location);
    }

    true
}

fn jump_to_bookmark(index: usize, state: &mut State, b: &ReadStorage<Bookmark>, pb: &WriteStorage<Position2D>) -> bool {
    if let Some((_, pos)) = (b, pb).join().find(|(bm, _)| bm.0 == index) {
        state.offset = *pos;
    } else {
        state.set_error(Error::execution("Bookmark not found"));
    }

    false
}

fn clear_blank_texels(state: &mut State, sp: &mut WriteStorage<Sprite>, s: &ReadStorage<Selection>) -> bool {
    if s.count() == 0 {
        return state.set_error(Error::execution("No objects selected"));
    }

    use crate::common::SpriteExt;

    let mut changed = false;
    for (sprite, _) in (sp, s).join() {
        sprite.clear_blank_texels(None);
        changed = true;
    }

    changed
}

fn apply_layout_to_selected(
    layout: Layout,
    state: &State,
    p: &mut WriteStorage<Position>,
    d: &WriteStorage<Dimension>,
    s: &ReadStorage<Selection>,
) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let bounds = viewport_bounds(state).unwrap(); // this should never be empty

    match layout {
        Layout::None => {}
        Layout::Column(cols, padding) => {
            let mut col_sizes = [0i32].repeat(cols);
            let mut row_sizes = Vec::new();
            let mut start_x = i32::max_value();
            let mut start_y = i32::max_value();
            let mut positions: Vec<&mut Position> = Vec::new();

            for (i, (pos, dim, _)) in (p, d, s).join().enumerate() {
                let col = i % cols;
                let row = i / cols;

                if row_sizes.len() <= row {
                    row_sizes.push(0i32);
                }

                col_sizes[col] = std::cmp::max(col_sizes[col], i32::from(dim.w));
                row_sizes[row] = std::cmp::max(row_sizes[row], i32::from(dim.h));

                if pos.x < start_x {
                    start_x = pos.x;
                }
                if pos.y < start_y {
                    start_y = pos.y;
                }

                positions.push(pos); // we can't re-iterate so keep the references
            }

            for (i, pos) in positions.iter_mut().enumerate() {
                let col = i % cols;
                let row = i / cols;

                let offset_x = (if col > 0 { col_sizes[col - 1] } else { 0 } + i32::from(padding.0)) * col as i32;
                let offset_y = (if row > 0 { row_sizes[row - 1] } else { 0 } + i32::from(padding.1)) * row as i32;

                pos.x = start_x + offset_x;
                pos.y = start_y + offset_y;
            }
        }
        Layout::Random => {
            for (pos, dim, _) in (p, d, s).join() {
                let bounds_x = bounds.position().x;
                let bounds_y = bounds.position().y;
                let bounds_w = i32::from(bounds.dimension().w);
                let bounds_h = i32::from(bounds.dimension().h);
                let dim_w = i32::from(dim.w);
                let dim_h = i32::from(dim.h);

                if dim_w < bounds_w && dim_h < bounds_h {
                    let x: i32 = rng.gen_range(bounds_x, bounds_x + bounds_w - dim_w);
                    let y: i32 = rng.gen_range(bounds_y, bounds_y + bounds_h - dim_h);
                    pos.x = x;
                    pos.y = y;
                }
            }
        }
    }

    false
}

fn change_frame_on_selected(
    which: Which<usize>,
    state: &mut State,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
) -> bool {
    if s.count() == 0 {
        return state.set_error(Error::execution("No objects selected"));
    }

    let mut changed = false;
    for (sprite, _) in (sp, s).join() {
        sprite.apply_frame_change(which);
        changed = true;
    }

    changed
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
    pss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;

    for (sprite, pos, _) in (sp, p, s).join() {
        if state.mode() == Mode::Edit {
            let sel_bounds = subselection(ss, pss, d).unwrap_or_else(|| Bounds::point(state.cursor));
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
    pss: &WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let bg = state.color(ColorMode::Bg);
    let fg = state.color(ColorMode::Fg);
    let sel_bounds = subselection(ss, pss, d).unwrap_or_else(|| Bounds::point(state.cursor));

    for (sprite, pos, dim, _) in (sp, p, d, s).join() {
        let pos2d: Position2D = pos.into();
        let rel_bounds = sel_bounds - pos2d;
        let bounds = sprite.apply_symbol(symbol, bg, fg, rel_bounds);

        clear_subselection(e, ss, u);
        // changed pos or dim => apply new bounds
        *pos += *bounds.position();
        *dim = *bounds.dimension();

        changed = true;
    }

    changed
}

fn clipboard(
    op: ClipboardOp,
    state: &mut State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    p: &mut WriteStorage<Position>,
    pss: &WriteStorage<Position2D>,
    d: &mut WriteStorage<Dimension>,
    u: &LazyUpdate,
) -> bool {
    match (state.mode(), op) {
        (Mode::Edit, ClipboardOp::Copy) => copy_or_cut_subselection(op, state, e, sp, s, ss, p, pss, d, u),
        (Mode::Edit, ClipboardOp::Cut) => copy_or_cut_subselection(op, state, e, sp, s, ss, p, pss, d, u),
        (Mode::Edit, ClipboardOp::Paste) => paste_subselection(state, sp, s, p, d),

        (Mode::Object(_), ClipboardOp::Copy) => copy_or_cut_selection(op, state, e, sp, s),
        (Mode::Object(_), ClipboardOp::Cut) => copy_or_cut_selection(op, state, e, sp, s),
        (Mode::Object(_), ClipboardOp::Paste) => paste_selection(state, e, s, u),
        _ => false,
    }
}

fn copy_or_cut_selection(
    op: ClipboardOp,
    state: &mut State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
) -> bool {
    let mut sprites: Vec<Sprite> = Vec::new();

    for (sprite, _) in (sp, s).join() {
        sprites.push(sprite.clone());
    }

    if sprites.is_empty() {
        return false;
    }

    state.clipboard = Clipboard::Sprites(sprites);

    if op == ClipboardOp::Cut {
        match delete_selected(e, s) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        }
    } else {
        false
    }
}

fn paste_selection(state: &mut State, e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) -> bool {
    let mut changed = false;
    let sprites: Vec<Sprite> = state.clipboard.clone().into();

    for sprite in sprites.into_iter() {
        if match import_sprite(sprite, state, e, s, u, None, true) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        } {
            changed = true;
        }
    }

    changed
}

fn copy_or_cut_subselection(
    op: ClipboardOp,
    state: &mut State,
    e: &Entities,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    ss: &WriteStorage<Subselection>,
    p: &mut WriteStorage<Position>,
    pss: &WriteStorage<Position2D>,
    d: &mut WriteStorage<Dimension>,
    u: &LazyUpdate,
) -> bool {
    let mut changed = false;
    let sel_bounds = subselection(ss, pss, d).unwrap_or_else(|| Bounds::point(state.cursor));

    if let Some((sprite, pos, dim, _)) = (sp, p, d, s).join().next() {
        let pos2d: Position2D = pos.into();
        let rel_bounds = sel_bounds - pos2d;

        state.clipboard = Clipboard::Texels(sprite.copy_area(rel_bounds));

        if op == ClipboardOp::Cut {
            changed = match sprite.clear_symbol(rel_bounds) {
                None => {
                    clear_subselection(e, ss, u);
                    false
                } // no change, symbol was applied in bounds
                Some(bounds) => {
                    // changed pos or dim => apply new bounds
                    *pos += *bounds.position();
                    *dim = *bounds.dimension();

                    clear_subselection(e, ss, u);
                    true
                }
            }
        } else {
            clear_subselection(e, ss, u);
        }
    }

    changed
}

fn paste_subselection(
    state: &mut State,
    sp: &mut WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    p: &mut WriteStorage<Position>,
    d: &mut WriteStorage<Dimension>,
) -> bool {
    if state.clipboard.is_empty() {
        return false;
    }

    let mut changed = false;

    if let Some((sprite, pos, dim, _)) = (sp, p, d, s).join().next() {
        let texels: Texels = state.clipboard.clone().into();
        let pos2d: Position2D = pos.into();
        let rel_pos = state.cursor - pos2d;

        let bounds = sprite.apply_texels(texels, rel_pos);
        *pos += *bounds.position();
        *dim = *bounds.dimension();

        changed = true
    }

    changed
}

fn new_sprite(state: &State, e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate, pos: Option<Position>) -> bool {
    deselect_obj(e, s, u);
    let entity = e.create();
    let sprite = Sprite::default();

    let dim = Dimension::for_sprite(&sprite);

    u.insert(entity, dim);
    u.insert(entity, pos.unwrap_or(NEW_POSITION + state.offset));
    u.insert(entity, Selection);
    u.insert(entity, Selectable);
    u.insert(entity, Border);
    u.insert(entity, sprite);

    true
}

fn duplicate_selected(
    count: usize,
    state: &State,
    e: &Entities,
    p: &WriteStorage<Position>,
    sp: &WriteStorage<Sprite>,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
) -> Result<(), Error> {
    let mut done = 0;
    for i in 0..count {
        let iteration = (i * 2) as i32;
        for (sprite, pos, _) in (sp, p, s).join() {
            done += 1;
            import_sprite(sprite.clone(), state, e, s, u, Some(*pos + 2 + iteration), true)?;
        }
    }

    if done > 0 {
        Ok(())
    } else {
        Err(Error::execution("Nothing to duplicate"))
    }
}

fn import_sprite(
    sprite: Sprite,
    state: &State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    u: &LazyUpdate,
    pos: Option<Position>,
    pre_select: bool,
) -> Result<(), Error> {
    deselect_obj(e, s, u);
    let entity = e.create();

    u.insert(entity, Dimension::for_sprite(&sprite));
    u.insert(entity, pos.unwrap_or(NEW_POSITION + state.offset));
    u.insert(entity, Selectable);
    u.insert(entity, Border);
    u.insert(entity, sprite);

    if pre_select {
        u.insert(entity, Selection);
    }

    Ok(())
}

fn save_scene(
    state: &mut State,
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    b: &ReadStorage<Bookmark>,
    pb: &WriteStorage<Position2D>,
    new_path: &Option<String>,
) -> Result<(), Error> {
    let path = state.save_file(new_path)?;
    let scene = Scene::from_runtime(sp, p, b, pb);

    fio::scene_to_file(&scene, &path)
}

fn export_to_file(
    format: ExportFormat,
    path: &str,
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    b: &ReadStorage<Bookmark>,
    pb: &WriteStorage<Position2D>,
) -> Result<(), Error> {
    let scene = Scene::from_runtime(sp, p, b, pb);

    fio::export_to_file(scene, format, path)
}

fn tutorial(
    e: &Entities,
    state: &mut State,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    b: &ReadStorage<Bookmark>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> Result<bool, Error> {
    if state.unsaved_changes() {
        Err(Error::execution("Unsaved changes, save before opening tutorial"))
    } else {
        use fio::Loaded;
        let bytes = include_bytes!("../../help/tutorial.rgz");
        let scene = match fio::scene_from_rgz_stream(&bytes[..])? {
            Loaded::Scene(scene) => scene,
            Loaded::Sprite(_) => return Err(Error::execution("Invalid const situation")),
        };

        apply_scene(scene.clone(), state, e, s, b, sp, pb, u, None)?;
        state.clear_history(scene); // we're going from this scene now
        state.reset_save_file(); // ensure we don't save the tutorial into previous file
        Ok(false)
    }
}

fn load_from_file(
    e: &Entities,
    state: &mut State,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    b: &ReadStorage<Bookmark>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
    path: &str,
) -> Result<bool, Error> {
    use fio::Loaded;

    match fio::load_from_file(path)? {
        Loaded::Scene(scene) => {
            if state.unsaved_changes() {
                Err(Error::execution("Unsaved changes, save before opening another scene"))
            } else {
                apply_scene(scene.clone(), state, e, s, b, sp, pb, u, None)?;
                state.clear_history(scene); // we're going from this scene now
                state.saved(String::from(path));
                Ok(false)
            }
        }
        Loaded::Sprite(sprite) => {
            import_sprite(sprite, state, e, s, u, None, true)?;
            Ok(true)
        }
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
    state: &State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    b: &ReadStorage<Bookmark>,
    sp: &WriteStorage<Sprite>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
    selections: Option<Vec<usize>>,
) -> Result<(), Error> {
    clear_scene(e, sp)?;

    let current = scene.current();
    let selections = selections.unwrap_or_default();

    for (i, obj) in current.objects.into_iter().enumerate() {
        let selected = selections.contains(&i);
        import_sprite(obj.0, state, e, s, u, Some(obj.1), selected)?;
    }

    for (index, pos) in current.bookmarks.into_iter() {
        set_bookmark(index, pos, e, b, pb, u);
    }

    Ok(())
}

fn undo(
    state: &mut State,
    e: &Entities,
    s: &ReadStorage<Selection>,
    sp: &WriteStorage<Sprite>,
    b: &ReadStorage<Bookmark>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    if let Some(value) = state.undo() {
        match apply_scene(value.0, state, e, s, b, sp, pb, u, Some(value.1)) {
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
    b: &ReadStorage<Bookmark>,
    pb: &mut WriteStorage<Position2D>,
    u: &LazyUpdate,
) -> bool {
    if let Some(value) = state.redo() {
        match apply_scene(value.0, state, e, s, b, sp, pb, u, Some(value.1)) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        }
    } else {
        state.set_error(Error::execution("Nothing to redo"));
        false
    }
}
