use crate::common::{Scene, SceneExt};
use crate::components::{Position, Sprite};
use crate::resources::State;
use specs::{System, Write, WriteStorage};

pub struct HistoryHandler;

impl<'a> System<'a> for HistoryHandler {
    type SystemData = (Write<'a, State>, WriteStorage<'a, Position>, WriteStorage<'a, Sprite>);

    fn run(&mut self, (mut state, p, sp): Self::SystemData) {
        state.push_history(Scene::from_objects(&sp, &p));
    }
}
