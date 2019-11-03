use crate::resources::{State, SyncTerm};
use specs::System;
use std::io::Write;

pub struct ColorPaletteRenderer;

impl<'a> System<'a> for ColorPaletteRenderer {
    type SystemData = (specs::Write<'a, SyncTerm>, specs::Read<'a, State>);

    fn run(&mut self, (mut out, state): Self::SystemData) {
        if !state.mode().is_color_palette() {
            return;
        }

        let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
        let h = i32::from(ts.1);
        let min_x = crate::systems::PALETTE_OFFSET;
        let min_y = h - 14;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut count = 0;

        for x in min_x..min_x + 16 {
            for y in min_y..min_y + 14 {
                if count > 6 * 6 * 6 {
                    continue;
                }
                count += 1;

                write!(
                    out,
                    "{}{} ",
                    crate::common::goto(x, y),
                    termion::color::AnsiValue::rgb(r, g, b).bg_string(),
                )
                .unwrap();

                r += 1;
                if r > 5 {
                    r = 0;
                    g += 1;
                }

                if g > 5 {
                    g = 0;
                    b += 1;
                }

                if b > 5 {
                    b = 0;
                }
            }
        }
    }
}
