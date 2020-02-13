use crate::resources::{FrameBuffer, State};
use legion::prelude::World;

mod action_handler;
mod history_handler;
mod input_handler;
mod renderers;

pub use action_handler::handle_actions;
pub use history_handler::preserve_history;
pub use input_handler::handle_input;
pub use renderers::*;

pub struct TexelSystems;

impl TexelSystems {
    pub fn run(world: &mut World, state: &mut State, out: &mut FrameBuffer) {
        out.flip_buffers();

        handle_input(world, state);
        handle_actions(world, state);
        preserve_history(world, state);

        render_sprites(world, state, out);
        render_subselections(world, state, out);
        render_meta_info(world, state, out);
        render_cmdline(world, state, out);
    }
}
