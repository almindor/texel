use std::collections::BTreeMap;
use crate::components::{Bookmark, Sprite};
use specs::{Join, ReadStorage, WriteStorage};
use texel_types::{Position, Position2D};

pub use texel_types::{Scene, SceneV2};

pub trait SceneExt {
    fn from_runtime(
        sp: &WriteStorage<Sprite>,
        p: &WriteStorage<Position>,
        b: &ReadStorage<Bookmark>,
        pb: &WriteStorage<Position2D>,
    ) -> Scene;

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene;
}

impl SceneExt for Scene {
    fn from_runtime(
        sp: &WriteStorage<Sprite>,
        p: &WriteStorage<Position>,
        b: &ReadStorage<Bookmark>,
        pb: &WriteStorage<Position2D>,
    ) -> Scene {
        let mut objects = Vec::new();
        let mut bookmarks = BTreeMap::new();

        for (sprite, pos) in (sp, p).join() {
            objects.push((sprite.clone(), *pos));
        }

        for (bookmark, pos) in (b, pb).join() {
            bookmarks.insert(bookmark.0, *pos);
        }

        Scene::V2(SceneV2 { objects, bookmarks })
    }

    fn from_objects(objects: Vec<(Sprite, Position)>, bookmarks: BTreeMap<usize, Position2D>) -> Scene {
        Scene::V2(SceneV2 { objects, bookmarks })
    }
}
