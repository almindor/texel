use crate::components::Sprite;
use specs::{Join, WriteStorage};
use texel_types::Position;

pub use texel_types::{Scene, SceneV2};

pub trait SceneExt {
    fn from_runtime(sp: &WriteStorage<Sprite>, p: &WriteStorage<Position>) -> Scene;

    fn from_objects(objects: Vec<(Sprite, Position)>) -> Scene;
}

impl SceneExt for Scene {
    fn from_runtime(sp: &WriteStorage<Sprite>, p: &WriteStorage<Position>) -> Scene {
        let mut objects = Vec::new();

        for (sprite, pos) in (sp, p).join() {
            objects.push((sprite.clone(), *pos));
        }

        Scene::V2(SceneV2 { objects })
    }

    fn from_objects(objects: Vec<(Sprite, Position)>) -> Scene {
        Scene::V2(SceneV2 { objects })
    }
}
