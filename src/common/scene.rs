use crate::components::{Selection, Sprite};
use specs::{Entities, Join, ReadStorage, WriteStorage};
use texel_types::Position;

pub use texel_types::{Scene, SceneV1};

pub fn scene_from_objects(
    e: &Entities,
    sp: &WriteStorage<Sprite>,
    p: &WriteStorage<Position>,
    s: &ReadStorage<Selection>,
) -> Scene {
    let mut objects = Vec::new();

    for (entity, sprite, pos) in (e, sp, p).join() {
        objects.push((sprite.clone(), *pos, s.contains(entity)));
    }

    Scene::V1(SceneV1 { objects })
}
