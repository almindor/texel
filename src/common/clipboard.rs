use crate::components::Sprite;
use serde::{Deserialize, Serialize};
use texel_types::Texels;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardOp {
    Copy,
    Cut,
    Paste,
}

#[derive(Debug, Clone)]
pub enum Clipboard {
    Empty,
    Sprites(Vec<Sprite>),
    Texels(Texels),
}

impl From<Clipboard> for Texels {
    fn from(value: Clipboard) -> Self {
        match value {
            Clipboard::Empty => Texels::new(),
            Clipboard::Sprites(sprites) => sprites // get all texels from all sprites in their active frames
                .into_iter() // consume sprites so we don't need to clone here
                .flat_map(|s| s.into_iter()) // turn each sprite into iterator over active frame's texels
                .collect(), // collect all texels from resulting set into Vec<Texel>
            Clipboard::Texels(texels) => texels,
        }
    }
}

impl From<Clipboard> for Vec<Sprite> {
    fn from(value: Clipboard) -> Self {
        match value {
            Clipboard::Empty => Vec::new(),
            Clipboard::Sprites(sprites) => sprites,
            Clipboard::Texels(texels) => vec![Sprite::from_texels(texels)],
        }
    }
}

impl Clipboard {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}
