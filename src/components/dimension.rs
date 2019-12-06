use specs::{Component, VecStorage};

pub use crate::texel_types::Dimension; 

impl Component for Dimension {
    type Storage = VecStorage<Self>;
}
