use crate::common::Texel;
use crate::components::{Border, Dimension, Position, Selection, Sprite};
use crate::resources::{ColorPalette, SyncTerm};
use specs::{Entities, Join, ReadStorage, System};
use std::io::Write;

pub struct SpriteRenderer;

impl<'a> System<'a> for SpriteRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Dimension>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, Border>,
        ReadStorage<'a, Selection>,
    );

    fn run(&mut self, (mut out, e, p, d, s, b, sel): Self::SystemData) {
        let mut loc_info: Option<&Position> = None;

        // TODO: optimize using FlaggedStorage
        let mut sorted = (&e, &p, &d, &s).join().collect::<Vec<_>>();
        sorted.sort_by(|&a, &b| b.1.z.cmp(&a.1.z));

        for (entity, pos, dim, sprite) in sorted {
            write!(out, "{}", ColorPalette::default_fg()).unwrap();

            render_sprite(&mut out, &pos, &sprite);

            if b.contains(entity) && sel.contains(entity) {
                render_border(&mut out, &pos, *dim);
                loc_info = Some(pos);
            }
        }

        write!(out, "{}{}", ColorPalette::default_bg(), ColorPalette::default_fg()).unwrap();

        // location info status line
        if let Some(loc) = loc_info {
            let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
            let w = i32::from(ts.0);
            let h = i32::from(ts.1);
            write!(out, "{}{}", crate::common::goto(w - 10, h), loc).unwrap();
        }
    }
}

fn print_texel(out: &mut SyncTerm, p: &Position, t: &Texel) {
    write!(
        out,
        "{}{}{}{}",
        crate::common::goto(p.x + t.x, p.y + t.y),
        ColorPalette::u8_to_bg(t.bg),
        ColorPalette::u8_to_fg(t.fg),
        t.symbol,
    )
    .unwrap();
}

fn is_visible(x: i32, y: i32, ts: (u16, u16)) -> bool {
    let w = i32::from(ts.0);
    let h = i32::from(ts.1);

    x > 0 && x <= w && y > 0 && y <= h
}

fn render_sprite(out: &mut SyncTerm, p: &Position, s: &Sprite) {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise

    for t in s.texels.iter().filter(|t| p.x + t.x > 0 && p.y + t.y > 0) {
        if is_visible(p.x + t.x, p.y + t.y, ts) {
            print_texel(out, p, t);
        }
    }
}

fn render_border(out: &mut SyncTerm, p: &Position, d: Dimension) {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
    let min_x = p.x - 1;
    let min_y = p.y - 1;
    let b_w = i32::from(d.w + 1);
    let b_h = i32::from(d.h + 1);

    write!(out, "{}{}", ColorPalette::default_bg(), ColorPalette::default_fg()).unwrap();

    for y in min_y..=min_y + b_h {
        if y <= 0 {
            continue;
        }

        if y == min_y || y == min_y + b_h {
            let a = if y == min_y { '-' } else { '_' };
            for x in min_x..=min_x + b_w {
                if is_visible(x, y, ts) {
                    write!(out, "{}{}", crate::common::goto(x, y), a).unwrap();
                }
            }
        }

        if min_x + b_w <= i32::from(ts.0) {
            write!(out, "{}|", crate::common::goto(min_x + b_w, y)).unwrap();
        }

        if min_x > 0 {
            write!(out, "{}|", crate::common::goto(min_x, y)).unwrap();
        }
    }
}
