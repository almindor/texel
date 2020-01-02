use crate::components::{Sprite};
use specs::{Join, WriteStorage};
use texel_types::Position;

pub use texel_types::{Scene, SceneV2};

pub fn scene_from_objects(
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
) -> Scene {
    let mut objects = Vec::new();

    for (sprite, pos) in (sp, p).join() {
        objects.push((sprite.clone(), *pos));
    }

    Scene::V2(SceneV2 { objects })
}
