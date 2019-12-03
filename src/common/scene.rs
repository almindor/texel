use crate::components::{Position, Selection};
use crate::common::SpriteV1;
use serde::{Deserialize, Serialize};
use specs::{Entities, Join, ReadStorage, WriteStorage};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Scene {
    V1(SceneV1),
}

impl Default for Scene {
    fn default() -> Self {
        Scene::V1(SceneV1::default())
    }
}

impl Scene {
    pub fn current(self) -> SceneV1 {
        match self {
            Self::V1(scene) => scene,
            // TODO: once we have V2+ we'll need to return that and convert previous
        }
    }
}

// TODO: figure out a 0-copy way to keep scene serializable/deserializable
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SceneV1 {
    pub objects: Vec<(SpriteV1, Position, bool)>,
}

impl<'a>
    From<(
        &Entities<'a>,
        &WriteStorage<'a, SpriteV1>,
        &WriteStorage<'a, Position>,
        &ReadStorage<'a, Selection>,
    )> for SceneV1
{
    fn from(
        storage: (
            &Entities,
            &WriteStorage<'a, SpriteV1>,
            &WriteStorage<'a, Position>,
            &ReadStorage<'a, Selection>,
        ),
    ) -> Self {
        let mut objects = Vec::new();
        let (e, sp, p, s) = storage;

        for (entity, sprite, pos) in (e, sp, p).join() {
            objects.push((sprite.clone(), *pos, s.contains(entity)));
        }

        SceneV1 { objects }
    }
}
