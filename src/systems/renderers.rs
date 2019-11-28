use crate::resources::SyncTerm;
use specs::System;

mod cmdline_renderer;
mod color_palette_renderer;
mod sprite_renderer;
mod subselection_renderer;

pub use cmdline_renderer::CmdLineRenderer;
pub use color_palette_renderer::ColorPaletteRenderer;
pub use sprite_renderer::SpriteRenderer;
pub use subselection_renderer::SubselectionRenderer;

pub struct ClearScreen;

impl<'a> System<'a> for ClearScreen {
    type SystemData = specs::Write<'a, SyncTerm>;

    fn run(&mut self, mut out: Self::SystemData) {
        out.flip_buffers();
    }
}
