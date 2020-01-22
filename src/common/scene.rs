use std::collections::BTreeMap;
use std::ops::Deref;
use legion::prelude::*;
use crate::components::Bookmark;
pub use texel_types::{Scene, SceneV2, Sprite, Position, Position2D};

pub trait SceneExt {
    fn from_world(world: &mut World) -> Scene;

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene;
}

impl SceneExt for Scene {
    fn from_world(world: &mut World) -> Scene {
        let mut objects = Vec::new();
        let mut bookmarks = BTreeMap::new();

        let query = <(Read<Sprite>, Read<Position>)>::query();
        for (sprite, pos) in query.iter(world) {
            objects.push((sprite.deref().clone(), *pos));
        }

        let query = <(Read<Bookmark>, Read<Position2D>)>::query();
        for (bookmark, pos) in query.iter(world) {
            bookmarks.insert(bookmark.0, *pos);
        }

        Scene::V2(SceneV2 { objects, bookmarks })
    }

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene {
        Scene::V2(SceneV2 { objects, bookmarks })
    }
}
