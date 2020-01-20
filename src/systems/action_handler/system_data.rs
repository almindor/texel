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
    WriteStorage<'a, Position2D>, // cursor position, subselection etc.
    WriteStorage<'a, Dimension>,
    WriteStorage<'a, Sprite>,
    Read<'a, LazyUpdate>,
);

pub trait SystemDataExt<'a> {
    fn state(&mut self) -> &mut State;
    fn entities(&self) -> &Entities<'a>;
    fn positions(&mut self) -> &mut WriteStorage<'a, Position>;
    fn selectable(&self) -> &ReadStorage<'a, Selectable>;
    fn selected(&self) -> &ReadStorage<'a, Selection>;
    fn bookmarks(&self) -> &ReadStorage<'a, Bookmark>;
    fn subselections(&mut self) -> &mut WriteStorage<'a, Subselection>;
    fn positions_2d(&mut self) -> &mut WriteStorage<'a, Position2D>;
    fn dimensions(&mut self) -> &mut WriteStorage<'a, Dimension>;
    fn sprites(&mut self) -> &mut WriteStorage<'a, Sprite>;
    fn lazy_update(&self) -> &LazyUpdate;
}

impl<'a> SystemDataExt<'a> for SystemData<'a> {
    fn state(&mut self) -> &mut State {
        &mut self.0
    }

    fn entities(&self) -> &Entities<'a> {
        &self.1
    }

    fn positions(&mut self) -> &mut WriteStorage<'a, Position> {
        &mut self.2
    }

    fn selectable(&self) -> &ReadStorage<'a, Selectable> {
        &self.3
    }

    fn selected(&self) -> &ReadStorage<'a, Selection> {
        &self.4
    }

    fn bookmarks(&self) -> &ReadStorage<'a, Bookmark> {
        &self.5
    }

    fn subselections(&mut self) -> &mut WriteStorage<'a, Subselection> {
        &mut self.6
    }

    fn positions_2d(&mut self) -> &mut WriteStorage<'a, Position2D> {
        &mut self.7
    }

    fn dimensions(&mut self) -> &mut WriteStorage<'a, Dimension> {
        &mut self.8
    }

    fn sprites(&mut self) -> &mut WriteStorage<'a, Sprite> {
        &mut self.9
    }

    fn lazy_update(&self) -> &LazyUpdate {
        &self.10
    }
}
