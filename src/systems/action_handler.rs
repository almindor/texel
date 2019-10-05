use crate::common::{cwd_path, Action, Error, Scene};
use crate::components::*;
use crate::resources::{ColorMode, Loaded, Loader, State};
use libflate::gzip::Encoder;
use specs::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
use std::path::{Path, PathBuf};

pub struct ActionHandler;

impl ActionHandler {
    fn deselect(e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) -> bool {
        let mut changed = false;
        for (entity, _) in (e, s).join() {
            u.remove::<Selection>(entity);
            changed = true;
        }

        changed
    }

    fn select_next(
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
            Self::deselect(e, s, u);
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
            if let Err(_) = e.delete(entity) {
                return Err(Error::execution("Error deleting entity"));
            } else {
                deleted += 1;
            }
        }

        if deleted == 0 {
            return Err(Error::execution("No entity to delete"));
        }

        Ok(())
    }

    fn translate_selected(
        t: Translation,
        p: &mut WriteStorage<Position>,
        s: &ReadStorage<Selection>,
        d: &ReadStorage<Dimension>,
    ) -> bool {
        let mut changed = false;

        for (position, _, dimension) in (p, s, d).join() {
            position.apply(t, dimension.w, dimension.h);
            changed = true;
        }

        changed
    }

    fn apply_color_to_selected(
        cm: ColorMode,
        state: &State,
        sp: &mut WriteStorage<Sprite>,
        s: &ReadStorage<Selection>,
    ) -> bool {
        let mut changed = false;
        let color = state.color(cm);

        for (sprite, _) in (sp, s).join() {
            sprite.fill(cm, color);
            changed = true;
        }

        changed
    }

    fn import_sprite(
        sprite: Sprite,
        e: &Entities,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
        pos: Option<Position>,
        pre_select: bool,
    ) -> Result<(), Error> {
        Self::deselect(e, s, u);
        let entity = e.create();

        u.insert(entity, Dimension::for_sprite(&sprite)?);
        u.insert(entity, pos.unwrap_or(Position::from_xyz(10, 10, 0)));
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
        sp: &WriteStorage<Sprite>,
        p: &WriteStorage<Position>,
        s: &ReadStorage<Selection>,
        path: &str,
    ) -> Result<(), Error> {
        let ronified = ron::ser::to_string(&Scene::from((e, sp, p, s)))?;
        let raw_path = if Path::new(path).extension() != Some(std::ffi::OsStr::new("rgz")) {
            Path::new(path).with_extension("rgz")
        } else {
            PathBuf::from(path)
        };
        let abs_path = cwd_path(&raw_path)?;

        let file = std::fs::File::create(abs_path)?;
        let mut encoder = Encoder::new(file)?;

        use std::io::Write;
        encoder.write(ronified.as_ref())?;
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
            Loaded::Scene(scene) => Self::apply_scene(scene, e, s, sp, u),
            Loaded::Sprite(sprite) => Self::import_sprite(sprite, e, s, u, None, true),
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
        Self::clear_scene(e, sp)?;

        for obj in scene.objects {
            Self::import_sprite(obj.0, e, s, u, Some(obj.1), obj.2)?;
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
            return match Self::apply_scene(scene, &e, &s, &sp, &u) {
                Ok(_) => true,
                Err(err) => state.set_error(err),
            };
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
            return match Self::apply_scene(scene, &e, &s, &sp, &u) {
                Ok(_) => true,
                Err(err) => state.set_error(err),
            };
        } else {
            state.set_error(Error::execution("Nothing to redo"));
            false
        }
    }
}

impl<'a> System<'a> for ActionHandler {
    type SystemData = (
        Entities<'a>,
        Write<'a, State>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selectable>,
        ReadStorage<'a, Selection>,
        ReadStorage<'a, Dimension>,
        WriteStorage<'a, Sprite>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (e, mut state, mut p, sel, s, d, mut sp, u): Self::SystemData) {
        while let Some(action) = state.pop_action() {
            let keep_history = action.keeps_history();

            let changed = match action {
                Action::None => false,
                Action::Undo => Self::undo(&mut state, &e, &s, &sp, &u),
                Action::Redo => Self::redo(&mut state, &e, &s, &sp, &u),
                Action::ClearError => state.clear_error(),
                Action::SetMode(mode) => state.set_mode(mode),
                Action::ApplyColor(cm) => Self::apply_color_to_selected(cm, &state, &mut sp, &s),
                Action::ReverseMode => state.reverse_mode(),
                Action::Deselect => Self::deselect(&e, &s, &u),
                Action::SelectNext(keep) => Self::select_next(&e, &sel, &s, &u, keep),
                Action::Translate(t) => Self::translate_selected(t, &mut p, &s, &d),
                Action::Delete => {
                    if let Err(err) = Self::delete_selected(&e, &s) {
                        state.set_error(err)
                    } else {
                        true
                    }
                }
                Action::Import(sprite) => {
                    if let Err(err) = Self::import_sprite(sprite, &e, &s, &u, None, true) {
                        state.set_error(err)
                    } else {
                        true
                    }
                }
                Action::Write(path) => {
                    if let Err(err) = Self::save_scene(&e, &sp, &p, &s, &path) {
                        state.set_error(err)
                    } else {
                        state.saved()
                    }
                }
                Action::Read(path) => {
                    if let Err(err) = Self::load_from_file(&e, &s, &sp, &u, &path) {
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
