use crate::common::{Scene, SceneExt};
use crate::components::{Position, Selection, Sprite};
use crate::resources::State;
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

pub struct HistoryHandler;

impl<'a> System<'a> for HistoryHandler {
    type SystemData = (
        Write<'a, State>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Sprite>,
        ReadStorage<'a, Selection>,
    );

    fn run(&mut self, (mut state, e, p, sp, s): Self::SystemData) {
        if !state.dirty {
            return;
        }

        let mut objects = Vec::new();
        let mut selections = Vec::new();

        for (i, (entity, pos, sprite)) in (&e, &p, &sp).join().enumerate() {
            objects.push((sprite.clone(), *pos));
            if s.contains(entity) {
                selections.push(i);
            }
        }

        let scene = Scene::from_objects(objects);
        state.push_history(scene, selections);
    }
}
