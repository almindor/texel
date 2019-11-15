use crate::components::Position2D;
use crate::resources::{Mode, State, SyncTerm};
use specs::System;
use std::io::Write;

pub struct ColorPaletteRenderer;

const MAX_COLOR_INDEX: u8 = 6 * 6 * 6;
const PALETTE_W: i32 = 16;
const PALETTE_H: i32 = 14;
const PALETTE_OFFSET: i32 = 24;

impl<'a> System<'a> for ColorPaletteRenderer {
    type SystemData = (specs::Write<'a, SyncTerm>, specs::Read<'a, State>);

    fn run(&mut self, (mut out, state): Self::SystemData) {
        match state.mode() {
            Mode::SelectColor(i) => print_palette(&mut out, &state, i),
            _ => {}
        }
    }
}

fn print_palette(out: &mut SyncTerm, state: &State, index: usize) {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    let h = i32::from(ts.1);
    let min = Position2D { x: crate::systems::PALETTE_OFFSET, y: h - PALETTE_H };
    let mut count = 0;

    for y in min.y..min.y + PALETTE_H {
        for x in min.x..min.x + PALETTE_W {
            if count >= MAX_COLOR_INDEX {
                break;
            }

            let (r, g, b) = base_to_rgb(count);
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

    let normalized_cursor = state.cursor - min;
    let base = pos_to_base(normalized_cursor);
    if base >= MAX_COLOR_INDEX {
        return;
    }

    let (r, g, b) = base_to_rgb(base);

    write!(
        out,
        "{}{}{}",
        crate::common::goto(PALETTE_OFFSET + index as i32, h),
        termion::color::AnsiValue::rgb(r, g, b).bg_string(),
        crate::common::index_from_one(index),
    )
    .unwrap();
}

const fn base_to_rgb(base: u8) -> (u8, u8, u8) {
    (base / 36, (base / 6) % 6, base % 6)
}

const fn pos_to_base(pos: Position2D) -> u8 {
    (pos.y * PALETTE_W) as u8 + pos.x as u8
}
