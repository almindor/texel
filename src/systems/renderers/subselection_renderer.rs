use crate::common::Mode;
use crate::components::{Dimension, Position2D, Subselection};
use crate::resources::{ColorPalette, State, SyncTerm};
use specs::{Join, Read, ReadStorage, System, Write};
use texel_types::{SymbolStyles, Texel};

pub struct SubselectionRenderer;

impl<'a> System<'a> for SubselectionRenderer {
    type SystemData = (
        Write<'a, SyncTerm>,
        Read<'a, State>,
        ReadStorage<'a, Position2D>,
        ReadStorage<'a, Dimension>,
        ReadStorage<'a, Subselection>,
    );

    fn run(&mut self, (mut out, state, p, d, ss): Self::SystemData) {
        if state.mode() != Mode::Edit {
            return;
        }

        let select_color = ColorPalette::subselection_bg_u8();

        for (pos, dim, _) in (&p, &d, &ss).join() {
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
}
