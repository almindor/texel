use crate::common::{Mode, SelectMode};
use crate::components::{Dimension, Position2D, Subselection};
use crate::resources::{ColorPalette, FrameBuffer, State};
use legion::prelude::*;
use texel_types::{SymbolStyles, Texel};

pub fn render_subselections(world: &mut World, state: &mut State, out: &mut FrameBuffer) {
    match state.mode() {
        Mode::Edit | Mode::Object(SelectMode::Region) => {}
        _ => return,
    }

    let select_color = ColorPalette::subselection_bg_u8();

    let query = <(Read<Position2D>, Read<Dimension>)>::query().filter(tag::<Subselection>());

    for (pos, dim) in query.iter(world) {
        let texels = Position2D::area_texels(*pos, *dim);

        for pos in texels {
            out.override_texel_bg(Texel {
                pos,
                symbol: ' ',
                bg: select_color,
                fg: texel_types::DEFAULT_FG_U8,
                styles: SymbolStyles::new(),
            });
        }
    }
}
