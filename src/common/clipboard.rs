use serde::{Serialize, Deserialize};
use crate::common::Texels;
use crate::components::Sprite;

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

impl Into<Texels> for Clipboard {
    fn into(self) -> Texels {
        match self {
            Self::Empty => Texels::new(),
            Self::Sprites(sprites) => sprites // get all texels from all sprites in their active frames
                                        .into_iter() // consume sprites so we don't need to clone here
                                        .map(|s| s.into_iter()) // turn each sprite into iterator over active frame's texels
                                        .flatten() // flatten the resulting vector of texel vectors into single
                                        .collect(), // collect all texels from resulting set into Vec<Texel>
            Self::Texels(texels) => texels,
        }
    }
}

impl Into<Vec<Sprite>> for Clipboard {
    fn into(self) -> Vec<Sprite> {
        match self {
            Self::Empty => Vec::new(),
            Self::Sprites(sprites) => sprites,
            Self::Texels(texels) => vec!(Sprite::from_texels(texels)),
        }
    }
}

impl Clipboard {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
    }
}