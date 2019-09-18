use crate::common::{Action, ExecuteError};
use crate::components::*;
use crate::resources::State;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
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
        let all: Vec<&Selectable> = sel.join().collect();

        for (entity, _) in (e, sel).join() {
            if s.get(entity).is_some() {

            }
        }

        if !keep {
            self.deselect(e, s, u);
        }
    }

    fn delete_selected(
        &mut self,
        e: &Entities,
        s: &ReadStorage<Selection>,
    ) -> Option<ExecuteError> {
        let mut deleted = 0usize;

        for (entity, _) in (e, s).join() {
            if let Err(_) = e.delete(entity) {
                return Some(ExecuteError::ExecutionError(String::from(
                    "Error deleting entity",
                )));
            } else {
                deleted += 1;
            }
        }

        if deleted == 0 {
            return Some(ExecuteError::ExecutionError(String::from(
                "No entity to delete",
            )));
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
        path: &Path,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
    ) -> Result<(), ExecuteError> {
        self.deselect(e, s, u);
        let entity = e.create();

        let sprite = Sprite::from_file(path)?;

        u.insert(entity, Dimension::for_sprite(&sprite)?);
        u.insert(entity, Position::from_xy(15, 13)); // TODO
        u.insert(entity, Color::default());
        u.insert(entity, Selection);
        u.insert(entity, Selectable);
        u.insert(entity, Border);
        u.insert(entity, sprite);

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
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (e, mut state, mut p, sel, s, d, u): Self::SystemData) {
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
                Action::Import(path) => {
                    if let Err(err) = self.import_sprite(&e, &path, &s, &u) {
                        state.set_error(Some(err));
                    }
                }
            }
        }
    }
}
