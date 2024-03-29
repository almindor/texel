use crate::components::Bookmark;
use legion::*;
use std::collections::BTreeMap;
pub use texel_types::{Position, Position2D, Scene, SceneV2, Sprite};

pub trait SceneExt {
    fn from_world(world: &mut World) -> Scene;

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene;
}

impl SceneExt for Scene {
    fn from_world(world: &mut World) -> Scene {
        let mut objects = Vec::new();
        let mut bookmarks = BTreeMap::new();

        let mut query = <(Read<Sprite>, Read<Position>)>::query();
        for (sprite, pos) in query.iter(world) {
            objects.push((sprite.clone(), *pos));
        }

        let mut query = <(Read<Bookmark>, Read<Position2D>)>::query();
        for (bookmark, pos) in query.iter(world) {
            bookmarks.insert(bookmark.0, *pos);
        }

        Scene::V2(SceneV2 { objects, bookmarks })
    }

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene {
        Scene::V2(SceneV2 { objects, bookmarks })
    }
}
