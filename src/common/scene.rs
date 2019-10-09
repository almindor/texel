use crate::components::{Position, Selection, Sprite};
use serde::{Deserialize, Serialize};
use specs::{Entities, Join, ReadStorage, WriteStorage};

// TODO: figure out a 0-copy way to keep scene serializable/deserializable
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Scene {
    pub objects: Vec<(Sprite, Position, bool)>,
}

impl<'a>
    From<(
        &Entities<'a>,
        &WriteStorage<'a, Sprite>,
        &WriteStorage<'a, Position>,
        &ReadStorage<'a, Selection>,
    )> for Scene
{
    fn from(
        storage: (
            &Entities,
            &WriteStorage<'a, Sprite>,
            &WriteStorage<'a, Position>,
            &ReadStorage<'a, Selection>,
        ),
    ) -> Self {
        let mut objects = Vec::new();
        let (e, sp, p, s) = storage;

        for (entity, sprite, pos) in (e, sp, p).join() {
            objects.push((sprite.clone(), pos.clone(), s.contains(entity)));
        }

        Scene { objects }
    }
}
