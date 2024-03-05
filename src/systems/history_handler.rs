use crate::common::{Scene, SceneExt};
use crate::components::{Bookmark, Position, Position2D, Selection, Sprite};
use crate::resources::State;
use legion::*;
use std::collections::BTreeMap;

pub fn preserve_history(world: &mut World, state: &mut State) {
    if !state.needs_preserving() {
        return;
    }

    let mut objects = Vec::new();
    let mut selections = Vec::new();
    let mut bookmarks = BTreeMap::new();

    let mut query = <(Read<Position>, Read<Sprite>, TryRead<Selection>)>::query();

    for (i, (pos, sprite, selected)) in query.iter(world).enumerate() {
        objects.push((sprite.clone(), *pos));
        if selected.is_some() {
            selections.push(i);
        }
    }

    let mut query = <(Read<Position2D>, Read<Bookmark>)>::query();

    for (pos, bookmark) in query.iter(world) {
        bookmarks.insert(bookmark.0, *pos);
    }

    let scene = Scene::from_objects(objects, bookmarks);
    state.push_history(scene, selections);
}
