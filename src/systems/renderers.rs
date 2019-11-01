use crate::resources::SyncTerm;
use specs::System;
use std::io::Write;

mod cmdline_renderer;
mod sprite_renderer;
mod color_palette_renderer;

pub use cmdline_renderer::{CmdLineRenderer, PALETTE_OFFSET};
pub use sprite_renderer::SpriteRenderer;
pub use color_palette_renderer::ColorPaletteRenderer;

pub struct ClearScreen;

impl<'a> System<'a> for ClearScreen {
    type SystemData = specs::Write<'a, SyncTerm>;

    fn run(&mut self, mut out: Self::SystemData) {
        write!(out, "{}", termion::clear::All).unwrap();
    }
}
