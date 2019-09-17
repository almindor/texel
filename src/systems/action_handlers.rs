use crate::common::Action;
use crate::components::{Dimension, Position, Selection, Translation};
use crate::resources::{State, ExecuteError};
use specs::{ReadStorage, System, Write, WriteStorage, Entities, Join};

pub struct ActionHandler;

impl ActionHandler {
    fn delete_selected(&mut self, entities: &Entities, s: &ReadStorage<Selection>) -> Option<ExecuteError> {
        let mut deleted = 0usize;

        for (e, _selection) in (entities, s).join() {
            if let Err(_) = entities.delete(e) {
                return Some(ExecuteError::ExecutionError("Error deleting entity"));
            } else {
                deleted += 1;
            }
        }

        if deleted == 0 {
            return Some(ExecuteError::ExecutionError("No entity to delete"))
        }

        None
    }

    fn translate_selected(&mut self, t: Translation, p: &mut WriteStorage<Position>, s: &ReadStorage<Selection>, d: &ReadStorage<Dimension>) {
        for (position, _selection, dimension) in (p, s, d).join() {
            position.apply(t, dimension.w, dimension.h);
        }
    }
}

impl<'a> System<'a> for ActionHandler {
    type SystemData = (
        Entities<'a>,
        Write<'a, State>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selection>,
        ReadStorage<'a, Dimension>,
    );

    fn run(&mut self, (entities, mut state, mut p, s, d): Self::SystemData) {
        while let Some(action) = state.pop_action() {
            match action {
                Action::None => {}
                Action::SetMode(mode) => state.set_mode(mode),
                Action::ReverseMode => state.reverse_mode(),
                Action::Translate(translation) => self.translate_selected(translation, &mut p, &s, &d),
                Action::Delete => state.error = self.delete_selected(&entities, &s),
            }
        }
    }
}
