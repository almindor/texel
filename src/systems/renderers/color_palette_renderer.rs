use crate::components::Position2D;
use crate::resources::{
    ColorMode, ColorPalette, Mode, State, SyncTerm, MAX_COLOR_INDEX, PALETTE_H, PALETTE_OFFSET, PALETTE_W,
};
use specs::System;
use std::io::Write;

pub struct ColorPaletteRenderer;

impl<'a> System<'a> for ColorPaletteRenderer {
    type SystemData = (specs::Write<'a, SyncTerm>, specs::Read<'a, State>);

    fn run(&mut self, (mut out, state): Self::SystemData) {
        if let Mode::SelectColor(i, cm) = state.mode() {
            print_palette(&mut out, &state, i, cm)
        }
    }
}

fn print_palette(out: &mut SyncTerm, state: &State, index: usize, cm: ColorMode) {
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

            write!(
                out,
                "{}{} ",
                crate::common::goto(x, y),
                termion::color::AnsiValue::rgb(r, g, b).bg_string(),
            )
            .unwrap();
        }
    }

    let color = ColorPalette::pos_to_color(state.cursor);

    let color_string = match cm {
        ColorMode::Fg => ColorPalette::u8_to_fg_string(color),
        ColorMode::Bg => ColorPalette::u8_to_bg_string(color),
    };

    write!(
        out,
        "{}{}{}",
        crate::common::goto(PALETTE_OFFSET + index as i32, h),
        color_string,
        crate::common::index_from_one(index),
    )
    .unwrap();
}
