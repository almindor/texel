use crate::common::{
    fio, Action, Clipboard, ClipboardOp, Error, Layout, MetadataType, Mode, OnQuit, Scene, SceneExt, SelectMode,
};
use crate::components::*;
use crate::os::Terminal;
use crate::resources::{State, PALETTE_H, PALETTE_OFFSET, PALETTE_W};
use fio::ExportFormat;
use legion::prelude::*;
use texel_types::{ColorMode, SymbolStyle, Texels, Which};

const NEW_POSITION: Position = Position { x: 10, y: 10, z: 0 };

pub fn handle_actions(world: &mut World, state: &mut State) {
    while let Some(action) = state.pop_action() {
        if match action {
            Action::None => false,
            Action::Undo => undo(world, state),
            Action::Redo => redo(world, state),
            Action::Clipboard(op) => clipboard(op, world, state),
            Action::ToggleMetadata => toggle_metadata(state),
            Action::SetMetadata(mt) => set_metadata(mt, world, state),
            Action::NewObject => new_sprite(world, state, None),
            Action::Duplicate(count) => match duplicate_selected(count, world, state) {
                Ok(_) => true,
                Err(err) => state.set_error(err),
            },
            Action::NewFrame => new_frame_on_selected(world, state),
            Action::DeleteFrame => delete_frame_on_selected(world, state),
            Action::Bookmark(index, true) => set_bookmark(index, state.offset, world),
            Action::Bookmark(index, false) => jump_to_bookmark(index, world, state),
            Action::Cancel => cancel(world, state),
            Action::ClearError => state.clear_error(),
            Action::SetMode(mode) => set_mode(mode, world, state),
            Action::ApplyColor(cm) => apply_color_to_selected(cm, world, state),
            Action::ApplySymbol(sym) => apply_symbol_to_selected(sym, world, state),
            Action::ApplyStyle(style) => apply_style_to_selected(style, world, state),
            Action::ApplyRegion => apply_region(subselection(world), world, state),
            Action::ReverseMode => reverse_mode(world, state) && false, // NOTE: reverse returns if reverted, not dirty state
            Action::Deselect => clear_subselection(world) || deselect_obj(world),
            Action::Translate(t) => match state.mode() {
                Mode::Edit => {
                    let sprite_bounds = selected_bounds(world);
                    translate_subselection(t, sprite_bounds, world, state)
                }
                Mode::Object(SelectMode::Region) => {
                    let viewport_bounds = viewport_bounds(&state);
                    translate_subselection(t, Some(viewport_bounds), world, state)
                }
                _ => translate_selected(t, world, state),
            },
            Action::Layout(layout) => apply_layout_to_selected(layout, world, state),
            Action::SelectFrame(which) => change_frame_on_selected(which, world, state),
            Action::SelectObject(which, sticky) => select_obj(which, sticky, world, state),
            Action::SelectRegion => select_region(world, state),
            Action::Delete => {
                if state.mode() == Mode::Edit || state.mode() == Mode::Write {
                    clear_symbol_on_selected(world, state)
                } else if let Err(err) = delete_selected(world) {
                    state.set_error(err)
                } else {
                    true
                }
            }
            Action::Write(path) => {
                if let Err(err) = save_scene(&path, world, state) {
                    state.set_error(err)
                } else if let Some(path) = path {
                    state.saved(path)
                } else {
                    state.clear_changes()
                }
            }
            Action::Read(path) => match load_from_file(&path, world, state) {
                Ok(changed) => changed, // we reset history in some cases here
                Err(err) => state.set_error(err),
            },
            Action::Tutorial => match tutorial(world, state) {
                Ok(changed) => changed,
                Err(err) => state.set_error(err),
            },
            Action::Export(format, path) => {
                if let Err(err) = export_to_file(format, &path, world) {
                    state.set_error(err)
                } else {
                    false
                }
            }
            Action::ShowHelp(index) => {
                state.set_mode(Mode::Help(index));
                false
            }
            Action::ClearBlank => clear_blank_texels(world, state),
        } {
            state.dirty = true;
        }
    }
}

// NOTE: returns if mode was reverted, not if dirty
fn reverse_mode(world: &mut World, state: &mut State) -> bool {
    let modifies_cursor = state.mode().modifies_cursor();

    if state.reverse_mode() {
        if modifies_cursor {
            let todo = CommandBuffer::default();
            let query = <(Read<Position>, TryWrite<Position2D>)>::query().filter(component::<Selection>());
            for (entity, (pos, cur_pos)) in query.iter_entities(world) {
                let pos2d: Position2D = (*pos).into();
                if let Some(mut cp) = cur_pos {
                    *cp = state.cursor - pos2d; // update to last cursor position
                } else {
                    todo.add_component(entity, state.cursor - pos2d);
                }
            }

            todo.write(world);
        }

        true
    } else {
        clear_subselection(world)
    }
}

fn deselect_obj(world: &mut World) -> bool {
    let query = <Read<Selection>>::query();
    let todo = CommandBuffer::default();

    for (entity, _) in query.iter_entities(world) {
        todo.remove_component::<Selection>(entity);
    }

    todo.write(world);

    false
}

fn clear_subselection(world: &mut World) -> bool {
    let query = <Read<Subselection>>::query();
    let todo = CommandBuffer::default();

    if let Some((entity, _)) = query.iter_entities(world).next() {
        todo.delete(entity);
    };

    todo.write(world);

    false
}

fn set_mode(mode: Mode, world: &mut World, state: &mut State) -> bool {
    let mut dirty = false;

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
                if let Err(err) = save_scene(&None, world, state) {
                    state.set_error(err)
                } else {
                    true
                }
            } else {
                true
            }
        }
        Mode::Edit | Mode::Write => match <Read<Selection>>::query().iter(world).count() {
            1 => {
                state.clear_error();

                let query = <(Read<Position>, TryRead<Position2D>)>::query().filter(component::<Selection>());
                if let Some((pos, cur_pos)) = query.iter(world).next() {
                    let pos2d: Position2D = (*pos).into();

                    // if we're going from a non-cursory mode
                    if !state.mode().modifies_cursor() {
                        if let Some(cp) = cur_pos {
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
                dirty = new_sprite(world, state, None);
                dirty
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
            deselect_obj(world);
            true
        }
        _ => true,
    } && state.set_mode(mode)
    {
        clear_subselection(world);
    }

    dirty
}

fn cancel(world: &mut World, state: &mut State) -> bool {
    if state.error().is_some() {
        state.clear_error()
    } else if !reverse_mode(world, state) {
        deselect_obj(world)
    } else {
        false
    }
}

fn select_region(world: &mut World, state: &mut State) -> bool {
    match state.mode() {
        Mode::Edit => {
            mark_subselection(world, state);
            false
        }
        Mode::Object(SelectMode::Region) => apply_region(mark_subselection(world, state), world, state),
        _ => state.set_error(Error::execution("Region select in unexpected mode")),
    }
}

fn mark_subselection(world: &mut World, state: &State) -> Option<Bounds> {
    let todo = CommandBuffer::default();

    let clear_edit = |entity| {
        todo.delete(entity);
    };

    let new_edit = || {
        let pos = state.cursor;
        todo.insert((), vec![(pos, Dimension::unit(), Subselection::at(pos))]);
    };

    let query = <Write<Subselection>>::query();
    let result = if let Some((entity, mut sel)) = query.iter_entities(world).next() {
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
    };

    todo.write(world);

    result
}

fn apply_region(region: Option<Bounds>, world: &mut World, state: &mut State) -> bool {
    let area = match region {
        Some(bounds) => bounds,
        None => return false,
    };

    deselect_obj(world);

    let query = <(Read<Position>, Read<Dimension>)>::query().filter(tag::<Selectable>());
    let todo = CommandBuffer::default();
    for (entity, (pos, dim)) in query.iter_entities(world) {
        // any point inside region -> select
        let pos2d: Position2D = (*pos).into();
        if area.intersects(pos2d - state.offset, *dim) {
            todo.add_component(entity, Selection);
        }
    }

    todo.write(world);

    clear_subselection(world);
    state.reverse_mode();

    false
}

fn select_obj_relative(forward: bool, sticky: bool, world: &mut World, state: &State) -> bool {
    let mut all = Vec::new();
    let mut start = 0usize;
    let viewport = viewport_bounds(state);

    let query = <(Read<Position>, Read<Dimension>, TryRead<Selection>)>::query().filter(tag::<Selectable>());
    for (entity, (pos, _, selected)) in query.iter_entities(world).filter(|item| {
        let p = *((item.1).0);
        let d = *((item.1).1);
        viewport.intersects(p.into(), d)
    }) {
        all.push((entity, *pos, selected.is_some()));
    }

    // ensure we keep sorting order
    all.sort_by(|a, b| {
        let a_score = a.1.y * 100_000 + a.1.x * 100 + a.1.z;
        let b_score = b.1.y * 100_000 + b.1.x * 100 + b.1.z;

        if forward {
            a_score.cmp(&b_score)
        } else {
            b_score.cmp(&a_score)
        }
    });

    for (i, item) in all.iter().enumerate() {
        if item.2 && start < i {
            start = i
        }
    }

    // go through unselected items only, starting with last selection known
    let mut unselected_iter = all
        .iter()
        .cycle()
        .skip(start)
        .take(all.len())
        .filter(|(_, _, is_sel)| !is_sel);

    if !sticky {
        deselect_obj(world);
    }

    if let Some(entity) = unselected_iter.next() {
        world.add_component(entity.0, Selection);
    } else if let Some(entity) = all.first() {
        world.add_component(entity.0, Selection); // re-deselecting
    }

    false
}

fn select_obj_all(world: &mut World) -> bool {
    let todo = CommandBuffer::default();
    let query = <TryRead<Selection>>::query().filter(tag::<Selectable>());
    for (entity, selected) in query.iter_entities(world) {
        if selected.is_none() {
            todo.add_component(entity, Selection);
        }
    }

    todo.write(world);

    false
}

// fn select_obj_at(
//     pos: Position2D,
//     sticky: bool,
//     world: &mut World
// ) -> bool {
//     // TODO
//     false
// }

fn select_obj(which: Which<Position2D>, sticky: bool, world: &mut World, state: &State) -> bool {
    match which {
        Which::Next => select_obj_relative(true, sticky, world, state),
        Which::Previous => select_obj_relative(false, sticky, world, state),
        Which::All => select_obj_all(world),
        Which::At(_) => false,
        // Which::At(pos) => select_obj_at(pos, sticky, world),
    }
}

fn delete_selected(world: &mut World) -> Result<(), Error> {
    let mut deleted = 0usize;
    let todo = CommandBuffer::default();

    let query = <Read<Selection>>::query();
    for (entity, _) in query.iter_entities(world) {
        todo.delete(entity);
        deleted += 1;
    }

    if deleted == 0 {
        return Err(Error::execution("No entity to delete"));
    } else {
        todo.write(world);
    }

    Ok(())
}

fn viewport_bounds(state: &State) -> Bounds {
    let ts = Terminal::terminal_size();
    Bounds::Free(state.offset, Dimension::from_wh(ts.0, ts.1))
}

fn selected_bounds(world: &mut World) -> Option<Bounds> {
    let query = <(Read<Position>, Read<Dimension>)>::query().filter(component::<Selection>());
    if let Some((position, dim)) = query.iter(world).next() {
        Some(Bounds::Free((*position).into(), *dim))
    } else {
        None
    }
}

fn translate_subselection(t: Translation, area_bounds: Option<Bounds>, world: &mut World, state: &mut State) -> bool {
    if let Some(bounds) = area_bounds {
        if state.cursor.apply(t, bounds) {
            // if we have a subselection
            let query = <(Write<Position2D>, Write<Dimension>, Read<Subselection>)>::query();
            if let Some((mut pos, mut dim, sub_sel)) = query.iter(world).next() {
                if sub_sel.active {
                    // adjusting subselection
                    let edit_box = sub_sel.initial_pos.area(state.cursor);
                    *pos = *edit_box.position();
                    *dim = *edit_box.dimension();
                }
            }
        }
    }

    false
}

fn translate_selected(t: Translation, world: &mut World, state: &mut State) -> bool {
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
            if <Read<Selection>>::query().iter(world).count() == 0 {
                let screen_bounds = Bounds::Free(Position2D::default(), screen_dim);
                state.offset.apply(t, screen_bounds);
            }

            let query = <(Write<Position>, Read<Dimension>)>::query().filter(component::<Selection>());
            for (mut position, dim) in query.iter(world) {
                let sprite_bounds = Bounds::Free((*position).into(), *dim);
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

fn apply_color_to_selected(cm: ColorMode, world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let color = state.color(cm);
    let sel_bounds = subselection(world).unwrap_or_else(|| Bounds::point(state.cursor));

    let query = <(Write<Sprite>, Write<Position>)>::query().filter(component::<Selection>());
    for (mut sprite, pos) in query.iter(world) {
        if state.mode() == Mode::Edit {
            let pos2d: Position2D = (*pos).into();
            let rel_bounds = sel_bounds - pos2d;

            if (*sprite).apply_color(cm, color, rel_bounds) {
                changed = true;
            }
        } else if sprite.fill_color(cm, color) {
            changed = true;
        }
    }

    if state.mode() == Mode::Edit && changed {
        clear_subselection(world);
    }

    changed
}

fn clear_symbol_on_selected(world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let sel_bounds = subselection(world).unwrap_or_else(|| Bounds::point(state.cursor));

    let query = <(Write<Sprite>, Write<Position>, Write<Dimension>)>::query().filter(component::<Selection>());
    for (mut sprite, mut pos, mut dim) in query.iter(world) {
        let pos2d: Position2D = (*pos).into();
        let rel_bounds = sel_bounds - pos2d;

        match (*sprite).clear_symbol(rel_bounds) {
            None => {
                changed = true;
            } // no change, symbol was applied in bounds
            Some(bounds) => {
                // changed pos or dim => apply new bounds
                *pos += *bounds.position();
                *dim = *bounds.dimension();

                changed = true;
            }
        }
    }

    if changed {
        clear_subselection(world);
    }

    changed
}

fn subselection(world: &mut World) -> Option<Bounds> {
    let query = <(Read<Position2D>, Read<Dimension>)>::query().filter(component::<Subselection>());
    if let Some((pos, dim)) = query.iter(world).next() {
        Some(Bounds::Binding(*pos, *dim))
    } else {
        None
    }
}

fn new_frame_on_selected(world: &mut World, state: &mut State) -> bool {
    let mut changed = false;

    if <Read<Selection>>::query().iter(world).count() == 0 {
        state.set_error(Error::execution("No objects selected"));
        return false;
    }

    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    for mut sprite in query.iter(world) {
        sprite.new_frame();
        changed = true;
    }

    changed
}

fn delete_frame_on_selected(world: &mut World, state: &mut State) -> bool {
    let mut changed = false;

    if <Read<Selection>>::query().iter(world).count() == 0 {
        state.set_error(Error::execution("No objects selected"));
        return false;
    }

    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    for mut sprite in query.iter(world) {
        if sprite.delete_frame() {
            changed = true;
        }
    }

    changed
}

fn set_bookmark(index: usize, location: Position2D, world: &mut World) -> bool {
    let query = <(Write<Position2D>, Read<Bookmark>)>::query();
    if let Some((mut pos, _)) = query.iter(world).find(|(_, bm)| bm.0 == index) {
        if location != *pos {
            *pos = location;
        }

        return true;
    }

    world.insert((), vec![(Bookmark(index), location)]);

    true
}

fn jump_to_bookmark(index: usize, world: &mut World, state: &mut State) -> bool {
    let query = <(Read<Bookmark>, Read<Position2D>)>::query();
    if let Some((_, pos)) = query.iter(world).find(|(bm, _)| bm.0 == index) {
        state.offset = *pos;
    } else {
        state.set_error(Error::execution("Bookmark not found"));
    }

    false
}

fn clear_blank_texels(world: &mut World, state: &mut State) -> bool {
    use crate::common::SpriteExt;

    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    let mut changed = false;
    for mut sprite in query.iter(world) {
        sprite.clear_blank_texels(None);
        changed = true;
    }

    if !changed {
        state.set_error(Error::execution("No objects selected"))
    } else {
        changed
    }
}

fn apply_layout_to_selected(layout: Layout, world: &mut World, state: &mut State) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let bounds = viewport_bounds(state);

    match layout {
        Layout::None => {}
        Layout::Column(cols, padding) => {
            let mut col_sizes = [0i32].repeat(cols);
            let mut row_sizes = Vec::new();
            let mut start_x = i32::max_value();
            let mut start_y = i32::max_value();
            let mut positions = Vec::new();

            let query = <(Write<Position>, Read<Dimension>)>::query().filter(component::<Selection>());
            for (i, (pos, dim)) in query.iter(world).enumerate() {
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

            for (i, mut pos) in positions.into_iter().enumerate() {
                let col = i % cols;
                let row = i / cols;

                let offset_x = (if col > 0 { col_sizes[col - 1] } else { 0 } + i32::from(padding.0)) * col as i32;
                let offset_y = (if row > 0 { row_sizes[row - 1] } else { 0 } + i32::from(padding.1)) * row as i32;

                pos.x = start_x + offset_x;
                pos.y = start_y + offset_y;
            }
        }
        Layout::Random => {
            let query = <(Write<Position>, Read<Dimension>)>::query().filter(component::<Selection>());
            for (mut pos, dim) in query.iter(world) {
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

fn change_frame_on_selected(which: Which<usize>, world: &mut World, state: &mut State) -> bool {
    if <Read<Selection>>::query().iter(world).count() == 0 {
        return state.set_error(Error::execution("No objects selected"));
    }

    let mut changed = false;
    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    for mut sprite in query.iter(world) {
        sprite.apply_frame_change(which);
        changed = true;
    }

    changed
}

fn apply_style_to_selected(style: SymbolStyle, world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let sel_bounds = subselection(world).unwrap_or_else(|| Bounds::point(state.cursor));

    let query = <(Write<Sprite>, Read<Position>)>::query().filter(component::<Selection>());
    for (mut sprite, pos) in query.iter(world) {
        if state.mode() == Mode::Edit {
            let pos2d: Position2D = (*pos).into();
            let rel_bounds = sel_bounds - pos2d;

            if (*sprite).apply_style(style, rel_bounds) {
                changed = true;
            }
        } else if sprite.fill_style(style) {
            changed = true;
        }
    }

    if state.mode() == Mode::Edit && changed {
        clear_subselection(world);
    }

    changed
}

fn apply_symbol_to_selected(symbol: char, world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let bg = state.color(ColorMode::Bg);
    let fg = state.color(ColorMode::Fg);
    let sel_bounds = subselection(world).unwrap_or_else(|| Bounds::point(state.cursor));

    let query = <(Write<Sprite>, Write<Position>, Write<Dimension>)>::query().filter(component::<Selection>());
    for (mut sprite, mut pos, mut dim) in query.iter(world) {
        let pos2d: Position2D = (*pos).into();
        let rel_bounds = sel_bounds - pos2d;
        let bounds = (*sprite).apply_symbol(symbol, bg, fg, rel_bounds);

        // changed pos or dim => apply new bounds
        *pos += *bounds.position();
        *dim = *bounds.dimension();

        changed = true;
    }

    if changed {
        clear_subselection(world);
    }

    changed
}

fn clipboard(op: ClipboardOp, world: &mut World, state: &mut State) -> bool {
    match (state.mode(), op) {
        (Mode::Edit, ClipboardOp::Copy) => copy_or_cut_subselection(op, world, state),
        (Mode::Edit, ClipboardOp::Cut) => copy_or_cut_subselection(op, world, state),
        (Mode::Edit, ClipboardOp::Paste) => paste_subselection(world, state),

        (Mode::Object(_), ClipboardOp::Copy) => copy_or_cut_selection(op, world, state),
        (Mode::Object(_), ClipboardOp::Cut) => copy_or_cut_selection(op, world, state),
        (Mode::Object(_), ClipboardOp::Paste) => paste_selection(world, state),
        _ => false,
    }
}

fn copy_or_cut_selection(op: ClipboardOp, world: &mut World, state: &mut State) -> bool {
    let mut sprites: Vec<Sprite> = Vec::new();

    let query = <Read<Sprite>>::query().filter(component::<Selection>());
    for sprite in query.iter(world) {
        sprites.push((*sprite).clone());
    }

    if sprites.is_empty() {
        return false;
    }

    state.clipboard = Clipboard::Sprites(sprites);

    if op == ClipboardOp::Cut {
        match delete_selected(world) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        }
    } else {
        false
    }
}

fn paste_selection(world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let sprites: Vec<Sprite> = state.clipboard.clone().into();

    deselect_obj(world);
    for sprite in sprites.into_iter() {
        if match import_sprite(sprite, None, true, world, state) {
            Ok(_) => true,
            Err(err) => state.set_error(err),
        } {
            changed = true;
        }
    }

    changed
}

fn copy_or_cut_subselection(op: ClipboardOp, world: &mut World, state: &mut State) -> bool {
    let mut changed = false;
    let mut found = false;
    let sel_bounds = subselection(world).unwrap_or_else(|| Bounds::point(state.cursor));

    let query = <(Write<Sprite>, Write<Position>, Write<Dimension>)>::query().filter(component::<Selection>());
    if let Some((mut sprite, mut pos, mut dim)) = query.iter(world).next() {
        let pos2d: Position2D = (*pos).into();
        let rel_bounds = sel_bounds - pos2d;

        state.clipboard = Clipboard::Texels(sprite.copy_area(rel_bounds));

        if op == ClipboardOp::Cut {
            changed = match sprite.clear_symbol(rel_bounds) {
                None => false, // no change, symbol was applied in bounds
                Some(bounds) => {
                    // changed pos or dim => apply new bounds
                    *pos += *bounds.position();
                    *dim = *bounds.dimension();

                    true
                }
            }
        }
        found = true;
    }

    if found {
        clear_subselection(world);
    }

    changed
}

fn paste_subselection(world: &mut World, state: &mut State) -> bool {
    if state.clipboard.is_empty() {
        return false;
    }

    let mut changed = false;

    let query = <(Write<Sprite>, Write<Position>, Write<Dimension>)>::query().filter(component::<Selection>());
    if let Some((mut sprite, mut pos, mut dim)) = query.iter(world).next() {
        let texels: Texels = state.clipboard.clone().into();
        let pos2d: Position2D = (*pos).into();
        let rel_pos = state.cursor - pos2d;

        let bounds = sprite.apply_texels(texels, rel_pos);
        *pos += *bounds.position();
        *dim = *bounds.dimension();

        changed = true
    }

    changed
}

fn toggle_metadata(state: &mut State) -> bool {
    state.show_meta = !state.show_meta;

    false
}

fn set_metadata(mt: MetadataType, world: &mut World, state: &mut State) -> bool {
    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    let selected = query.iter(world).count();

    if selected == 0 {
        return state.set_error(Error::execution("No object selected"));
    }

    if mt.is_id() && selected > 1 {
        return state.set_error(Error::execution("Can only set ID on single object"));
    }

    let query = <Write<Sprite>>::query().filter(component::<Selection>());
    for mut sprite in query.iter(world) {
        match &mt {
            MetadataType::Id(id) => sprite.id = *id,
            MetadataType::Labels(labels) => sprite.labels.extend(labels.iter().map(|(k, v)| (k.clone(), v.clone()))),
        }
    }

    true
}

fn new_sprite(world: &mut World, state: &State, pos: Option<Position>) -> bool {
    deselect_obj(world);

    let sprite = Sprite::default();

    let dim = Dimension::for_sprite(&sprite);

    world.insert(
        (Selectable,),
        vec![(Selection, pos.unwrap_or(NEW_POSITION + state.offset), dim, sprite)],
    );

    true
}

fn duplicate_selected(count: usize, world: &mut World, state: &State) -> Result<(), Error> {
    let mut done = 0;
    let query = <(Read<Sprite>, Read<Position>)>::query().filter(component::<Selection>());
    let mut clones = Vec::new();

    for i in 0..count {
        let iteration = (i * 2) as i32;
        for (sprite, pos) in query.iter(world) {
            clones.push(((*sprite).clone(), Some(*pos + 2 + iteration)));
        }
    }

    deselect_obj(world);

    for (sprite, pos) in clones.into_iter() {
        import_sprite(sprite, pos, true, world, state)?;
        done += 1;
    }

    if done > 0 {
        Ok(())
    } else {
        Err(Error::execution("Nothing to duplicate"))
    }
}

fn import_sprite(
    sprite: Sprite,
    pos: Option<Position>,
    pre_select: bool,
    world: &mut World,
    state: &State,
) -> Result<(), Error> {
    if pre_select {
        world.insert(
            (Selectable,),
            vec![(
                Selection,
                pos.unwrap_or(NEW_POSITION + state.offset),
                Dimension::for_sprite(&sprite),
                sprite,
            )],
        );
    } else {
        world.insert(
            (Selectable,),
            vec![(
                pos.unwrap_or(NEW_POSITION + state.offset),
                Dimension::for_sprite(&sprite),
                sprite,
            )],
        );
    }

    Ok(())
}

fn save_scene(new_path: &Option<String>, world: &mut World, state: &mut State) -> Result<(), Error> {
    let path = state.save_file(new_path)?;
    let scene = Scene::from_world(world);

    fio::scene_to_file(&scene, &path)
}

fn export_to_file(format: ExportFormat, path: &str, world: &mut World) -> Result<(), Error> {
    let scene = Scene::from_world(world);

    fio::export_to_file(scene, format, path)
}

fn tutorial(world: &mut World, state: &mut State) -> Result<bool, Error> {
    if state.unsaved_changes() {
        Err(Error::execution("Unsaved changes, save before opening tutorial"))
    } else {
        use fio::Loaded;
        let bytes = include_bytes!("../../help/tutorial.rgz");
        let scene = match fio::scene_from_rgz_stream(&bytes[..])? {
            Loaded::Scene(scene) => scene,
            Loaded::Sprite(_) => return Err(Error::execution("Invalid const situation")),
        };

        apply_scene(scene.clone(), world, state, None)?;
        state.clear_history(scene); // we're going from this scene now
        state.reset_save_file(); // ensure we don't save the tutorial into previous file
        Ok(false)
    }
}

fn load_from_file(path: &str, world: &mut World, state: &mut State) -> Result<bool, Error> {
    use fio::Loaded;

    match fio::load_from_file(path)? {
        Loaded::Scene(scene) => {
            if state.unsaved_changes() {
                Err(Error::execution("Unsaved changes, save before opening another scene"))
            } else {
                apply_scene(scene.clone(), world, state, None)?;
                state.clear_history(scene); // we're going from this scene now
                state.saved(String::from(path));
                Ok(false)
            }
        }
        Loaded::Sprite(sprite) => {
            deselect_obj(world);
            import_sprite(sprite, None, true, world, state)?;
            Ok(true)
        }
    }
}

fn clear_scene(world: &mut World) -> Result<(), Error> {
    let todo = CommandBuffer::default();

    let query = <Tagged<Selectable>>::query();
    for (entity, _) in query.iter_entities(world) {
        todo.delete(entity);
    }

    todo.write(world);

    Ok(())
}

fn apply_scene(scene: Scene, world: &mut World, state: &State, selections: Option<Vec<usize>>) -> Result<(), Error> {
    clear_scene(world)?;

    let current = scene.current();
    let selections = selections.unwrap_or_default();

    for (i, obj) in current.objects.into_iter().enumerate() {
        let selected = selections.contains(&i);
        import_sprite(obj.0, Some(obj.1), selected, world, state)?;
    }

    for (index, pos) in current.bookmarks.into_iter() {
        set_bookmark(index, pos, world);
    }

    Ok(())
}

fn validate_mode(world: &mut World, state: &mut State) -> bool {
    let selected = <Read<Selection>>::query().iter(world).count();

    while !state.mode().valid_selection(selected) {
        state.reverse_mode(); // keep reversing until a valid mode for given selection
    }

    false
}

fn undo(world: &mut World, state: &mut State) -> bool {
    if let Some(snap) = state.undo() {
        match apply_scene(snap.scene, world, state, Some(snap.selections)) {
            Ok(_) => validate_mode(world, state),
            Err(err) => state.set_error(err),
        }
    } else {
        state.set_error(Error::execution("Nothing to undo"));
        false
    }
}

fn redo(world: &mut World, state: &mut State) -> bool {
    if let Some(snap) = state.redo() {
        match apply_scene(snap.scene, world, state, Some(snap.selections)) {
            Ok(_) => validate_mode(world, state),
            Err(err) => state.set_error(err),
        }
    } else {
        state.set_error(Error::execution("Nothing to redo"));
        false
    }
}
