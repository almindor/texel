use crate::resources::SyncTerm;
use specs::System;
use std::io::Write;

mod cmdline_renderer;
mod color_palette_renderer;
mod sprite_renderer;

pub use cmdline_renderer::{CmdLineRenderer, PALETTE_OFFSET};
pub use color_palette_renderer::ColorPaletteRenderer;
pub use sprite_renderer::SpriteRenderer;

pub struct ClearScreen;

impl<'a> System<'a> for ClearScreen {
    type SystemData = specs::Write<'a, SyncTerm>;

    fn run(&mut self, mut out: Self::SystemData) {
        write!(out, "{}", termion::clear::All).unwrap();
    }
}
