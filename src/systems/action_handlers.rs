use crate::common::{cwd_path, Action, Error, Scene};
use crate::components::*;
use crate::resources::State;
use specs::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
use std::path::Path;

pub struct ActionHandler;

impl ActionHandler {
    fn deselect(&mut self, e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) {
        for (entity, _) in (e, s).join() {
            u.remove::<Selection>(entity);
        }
    }

    fn select_next(
        &mut self,
        e: &Entities,
        sel: &ReadStorage<Selectable>,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
        keep: bool,
    ) {
        let mut all: Vec<(Entity, bool)> = Vec::default();
        let mut start = 0usize;

        for (i, (entity, _)) in (e, sel).join().enumerate() {
            let is_selected = s.get(entity).is_some();
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
            self.deselect(e, s, u);
        }

        if let Some(entity) = unselected_iter.next() {
            u.insert(entity.0, Selection); // select next if possible
        } else if let Some(entity) = all.first() {
            u.insert(entity.0, Selection); // select first if "redeselecting"
        }
    }

    fn delete_selected(&mut self, e: &Entities, s: &ReadStorage<Selection>) -> Option<Error> {
        let mut deleted = 0usize;

        for (entity, _) in (e, s).join() {
            if let Err(_) = e.delete(entity) {
                return Some(Error::ExecutionError(String::from("Error deleting entity")));
            } else {
                deleted += 1;
            }
        }

        if deleted == 0 {
            return Some(Error::ExecutionError(String::from("No entity to delete")));
        }

        None
    }

    fn translate_selected(
        &mut self,
        t: Translation,
        p: &mut WriteStorage<Position>,
        s: &ReadStorage<Selection>,
        d: &ReadStorage<Dimension>,
    ) {
        for (position, _, dimension) in (p, s, d).join() {
            position.apply(t, dimension.w, dimension.h);
        }
    }

    fn import_sprite(
        &mut self,
        e: &Entities,
        sprite: Sprite,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
    ) -> Result<(), Error> {
        self.deselect(e, s, u);
        let entity = e.create();

        u.insert(entity, Dimension::for_sprite(&sprite)?);
        u.insert(entity, Position::from_xy(15, 13)); // TODO
        u.insert(entity, Color::default());
        u.insert(entity, Selection);
        u.insert(entity, Selectable);
        u.insert(entity, Border);
        u.insert(entity, sprite);

        Ok(())
    }

    fn save_scene(&mut self, s: &ReadStorage<Sprite>, path: &str) -> Result<(), Error> {
        let ronified = ron::ser::to_string(&Scene::from(s))?;
        let abs_path = cwd_path(Path::new(path))?;

        std::fs::write(abs_path, ronified)?;
        Ok(())
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
        ReadStorage<'a, Sprite>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (e, mut state, mut p, sel, s, d, sp, u): Self::SystemData) {
        while let Some(action) = state.pop_action() {
            match action {
                Action::None => {}
                Action::ClearError => state.set_error(None),
                Action::SetMode(mode) => state.set_mode(mode),
                Action::ReverseMode => state.reverse_mode(),
                Action::Deselect => self.deselect(&e, &s, &u),
                Action::SelectNext(keep) => self.select_next(&e, &sel, &s, &u, keep),
                Action::Translate(t) => self.translate_selected(t, &mut p, &s, &d),
                Action::Delete => state.set_error(self.delete_selected(&e, &s)),
                Action::Import(sprite) => {
                    if let Err(err) = self.import_sprite(&e, sprite, &s, &u) {
                        state.set_error(Some(err));
                    }
                }
                Action::Save(path) => {
                    if let Err(err) = self.save_scene(&sp, &path) {
                        state.set_error(Some(err));
                    }
                }
                Action::Load(_) => {} // TODO
            }
        }
    }
}
