use crate::resources::{FrameBuffer, State};
use legion::*;

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
    pub fn run(world: &mut World, state: &mut State, resources: &mut Resources, out: &mut FrameBuffer) {
        out.flip_buffers();

        handle_input(state, resources);
        handle_actions(world, state);
        preserve_history(world, state);

        render_sprites(world, state, out);
        render_subselections(world, state, out);
        render_meta_info(world, state, out);
        render_cmdline(state, resources, out);
    }
}
