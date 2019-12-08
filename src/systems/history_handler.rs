use crate::common::scene_from_objects;
use crate::components::{Position, Selection, Sprite};
use crate::resources::State;
use specs::{Entities, ReadStorage, System, Write, WriteStorage};

pub struct HistoryHandler;

impl<'a> System<'a> for HistoryHandler {
    type SystemData = (
        Entities<'a>,
        Write<'a, State>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selection>,
        WriteStorage<'a, Sprite>,
    );

    fn run(&mut self, (e, mut state, p, s, sp): Self::SystemData) {
        state.push_history(scene_from_objects(&e, &sp, &p, &s));
    }
}
