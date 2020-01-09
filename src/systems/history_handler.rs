use std::collections::BTreeMap;
use crate::common::{Scene, SceneExt};
use crate::components::{Bookmark, Position, Position2D, Selection, Sprite};
use crate::resources::State;
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

pub struct HistoryHandler;

impl<'a> System<'a> for HistoryHandler {
    type SystemData = (
        Write<'a, State>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Sprite>,
        WriteStorage<'a, Position2D>,
        ReadStorage<'a, Selection>,
        ReadStorage<'a, Bookmark>,
    );

    fn run(&mut self, (mut state, e, p, sp, pb, s, b): Self::SystemData) {
        if !state.dirty {
            return;
        }

        let mut objects = Vec::new();
        let mut selections = Vec::new();
        let mut bookmarks = BTreeMap::new();

        for (i, (entity, pos, sprite)) in (&e, &p, &sp).join().enumerate() {
            objects.push((sprite.clone(), *pos));
            if s.contains(entity) {
                selections.push(i);
            }
        }

        for (bookmark, pos) in (&b, &pb).join() {
            bookmarks.insert(bookmark.0, *pos);
        }

        let scene = Scene::from_objects(objects, bookmarks);
        state.push_history(scene, selections);
    }
}
