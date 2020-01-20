use crate::components::*;
use crate::resources::State;
use specs::{Entities, LazyUpdate, Read, ReadStorage, Write, WriteStorage};

pub type SystemData<'a> = (
    Write<'a, State>,
    Entities<'a>,
    WriteStorage<'a, Position>,
    ReadStorage<'a, Selectable>,
    ReadStorage<'a, Selection>,
    ReadStorage<'a, Bookmark>,
    WriteStorage<'a, Subselection>,
    WriteStorage<'a, Position2D>, // cursor position saved to sprite, bookmark position
    WriteStorage<'a, Dimension>,
    WriteStorage<'a, Sprite>,
    Read<'a, LazyUpdate>,
);

pub trait SystemDataExt {
    fn state(&mut self) -> &mut State;
}

impl SystemDataExt for &mut SystemData<'_> {
    fn state(&mut self) -> &mut State {
        &mut self.0
    }
}