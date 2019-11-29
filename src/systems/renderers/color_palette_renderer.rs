use crate::common::SymbolStyles;
use crate::components::Position2D;
use crate::resources::{
    ColorMode, ColorPalette, Mode, State, SyncTerm, MAX_COLOR_INDEX, PALETTE_H, PALETTE_OFFSET, PALETTE_W,
};
use specs::System;

pub struct ColorPaletteRenderer;

type WorldInfo<'a> = (
    specs::Write<'a, SyncTerm>,
    specs::Read<'a, State>,
    specs::Read<'a, ColorPalette>,
);

impl<'a> System<'a> for ColorPaletteRenderer {
    type SystemData = WorldInfo<'a>;

    fn run(&mut self, world_info: Self::SystemData) {
        if let Mode::SelectColor(i, cm) = world_info.1.mode() {
            print_palette(world_info, i, cm)
        }
    }
}

fn print_palette(world_info: WorldInfo, index: usize, cm: ColorMode) {
    let (mut out, _, palette) = world_info;

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
                SymbolStyles::new(),
            );
        }
    }

    let x = PALETTE_OFFSET + (index as i32);
    out.write_texel(palette.selector_texel(index, x, h, cm));
}
