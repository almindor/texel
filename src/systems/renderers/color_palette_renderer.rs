use crate::components::Position2D;
use crate::resources::{
    ColorMode, ColorPalette, Mode, State, SyncTerm, MAX_COLOR_INDEX, PALETTE_H, PALETTE_OFFSET, PALETTE_W,
};
use big_enum_set::BigEnumSet;
use specs::System;

pub struct ColorPaletteRenderer;

impl<'a> System<'a> for ColorPaletteRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        specs::Read<'a, State>,
        specs::Read<'a, ColorPalette>,
    );

    fn run(&mut self, (mut out, state, palette): Self::SystemData) {
        if let Mode::SelectColor(i, cm) = state.mode() {
            print_palette(&mut out, i, cm, &palette)
        }
    }
}

fn print_palette(out: &mut SyncTerm, index: usize, cm: ColorMode, palette: &ColorPalette) {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    let h = i32::from(ts.1);
    let min = Position2D {
        x: PALETTE_OFFSET,
        y: h - PALETTE_H,
    };
    let mut count = 0;

    for y in min.y..min.y + PALETTE_H {
        for x in min.x..min.x + PALETTE_W {
            if count >= MAX_COLOR_INDEX {
                break;
            }

            let (r, g, b) = ColorPalette::base_to_rgb(count);
            count += 1;

            out.write_line(
                x,
                y,
                " ",
                termion::color::AnsiValue::rgb(r, g, b).0,
                ColorPalette::default_fg_u8(),
                BigEnumSet::new(),
            );
        }
    }

    let x = PALETTE_OFFSET + (index as i32);
    out.write_texel(palette.selector_texel(index, x, h, cm));
}
