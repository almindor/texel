use crate::texel_types::{Position, SpriteV1};
use serde::{Deserialize, Serialize};

// TODO: figure out a 0-copy way to keep scene serializable/deserializable
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SceneV1 {
    pub objects: Vec<(SpriteV1, Position, bool)>,
}
