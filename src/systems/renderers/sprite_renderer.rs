use crate::common::Texel;
use crate::components::{Border, Dimension, Position, Selection, Sprite};
use crate::resources::SyncTerm;
use specs::{Entities, Join, ReadStorage, System};

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
        let mut sprite_info: Option<(&Position, usize, usize)> = None;

        // TODO: optimize using FlaggedStorage
        let mut sorted = (&e, &p, &d, &s).join().collect::<Vec<_>>();
        sorted.sort_by(|&a, &b| b.1.z.cmp(&a.1.z));

        for (entity, pos, dim, sprite) in sorted {
            render_sprite(&mut out, &pos, &sprite);

            if b.contains(entity) && sel.contains(entity) {
                render_border(&mut out, &pos, *dim);
                sprite_info = Some((pos, sprite.frame_index(), sprite.frame_count()));
            }
        }

        // location info status line
        if let Some(si) = sprite_info {
            let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise
            let w = i32::from(ts.0);
            let h = i32::from(ts.1);
            let text = format!("[{}]::[{}/{}]", si.0, si.1, si.2);

            out.write_line_default(w - 15, h, text);
        }
    }
}

fn print_texel(out: &mut SyncTerm, p: &Position, t: &Texel) {
    let abs_texel = Texel {
        x: p.x + t.x,
        y: p.y + t.y,
        symbol: t.symbol,
        bg: t.bg,
        fg: t.fg,
        styles: t.styles,
    };

    out.write_texel(abs_texel);
}

fn is_visible(x: i32, y: i32, ts: (u16, u16)) -> bool {
    let w = i32::from(ts.0);
    let h = i32::from(ts.1);

    x > 0 && x <= w && y > 0 && y <= h
}

fn render_sprite(out: &mut SyncTerm, p: &Position, s: &Sprite) {
    let ts = termion::terminal_size().unwrap(); // this needs to panic since we lose output otherwise

    for t in s.frame_iter().filter(|t| p.x + t.x > 0 && p.y + t.y > 0) {
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

    for y in min_y..=min_y + b_h {
        if y <= 0 {
            continue;
        }

        if y == min_y || y == min_y + b_h {
            let a = if y == min_y { '-' } else { '_' };
            for x in min_x..=min_x + b_w {
                if is_visible(x, y, ts) {
                    out.write_line_default(x, y, a);
                }
            }
        }

        if min_x + b_w <= i32::from(ts.0) {
            out.write_line_default(min_x + b_w, y, "|");
        }

        if min_x > 0 {
            out.write_line_default(min_x, y, "|");
        }
    }
}
