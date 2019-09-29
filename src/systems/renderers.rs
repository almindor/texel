use crate::common::Texel;
use crate::components::{Border, ColorPalette, Dimension, Position, Selection, Sprite};
use crate::resources::{CmdLine, Mode, State, SyncTerm};
use specs::{Entities, Join, ReadStorage, System};
use std::io::Write;

pub struct ClearScreen;

pub struct SpriteRenderer;

pub struct CmdLineRenderer;

impl<'a> System<'a> for ClearScreen {
    type SystemData = specs::Write<'a, SyncTerm>;

    fn run(&mut self, mut out: Self::SystemData) {
        write!(out, "{}", termion::clear::All).unwrap();
    }
}

impl SpriteRenderer {
    fn print_texel(out: &mut SyncTerm, p: &Position, t: &Texel) {
        write!(
            out,
            "{}{}",
            crate::common::goto(p.x + t.x, p.y + t.y),
            t.symbol
        )
        .unwrap();
    }

    fn print_texel_symbol(out: &mut SyncTerm, p: &Position, t: &Texel) {
        write!(
            out,
            "{}{}{}",
            crate::common::goto(p.x + t.x, p.y + t.y),
            t.color,
            t.symbol,
        )
        .unwrap();
    }

    fn render_sprite(out: &mut SyncTerm, p: &Position, s: &Sprite) {
        let mut prev_color: Option<&str> = None;
        for t in s.texels.iter().filter(|t| p.x + t.x > 0 && p.y + t.y > 0) {
            if let Some(color) = prev_color {
                if color == t.color { // don't print same color needlessly
                    Self::print_texel_symbol(out, p, t);
                    continue;
                }
            }

            Self::print_texel(out, p, t);
            prev_color = Some(&t.color);
        }
    }

    fn render_border(out: &mut SyncTerm, p: &Position, d: &Dimension) {
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
                    write!(out, "{}{}", crate::common::goto(x, y), a).unwrap();
                }
            }

            write!(out, "{}|", crate::common::goto(min_x + b_w, y)).unwrap();

            if min_x <= 0 {
                continue;
            }

            write!(out, "{}|", crate::common::goto(min_x, y)).unwrap();
        }
    }
}

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

            Self::render_sprite(&mut out, &pos, &sprite);

            if b.contains(entity) && sel.contains(entity) {
                Self::render_border(&mut out, &pos, &dim);
                loc_info = Some(pos);
            }
        }

        write!(out, "{}", ColorPalette::default_fg()).unwrap();
        // location info status line
        if let Some(loc) = loc_info {
            let w = i32::from(out.w);
            let h = i32::from(out.h);
            write!(out, "{}{}", crate::common::goto(w - 10, h), loc).unwrap();
        }
    }
}

impl<'a> System<'a> for CmdLineRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        specs::Read<'a, State>,
        specs::Read<'a, CmdLine>,
    );

    fn run(&mut self, (mut out, state, cmdline): Self::SystemData) {
        let h = i32::from(out.h);

        if let Some(error) = state.error() {
            write!(
                out,
                "{}{}{}{}",
                crate::common::goto(1, h),
                termion::color::Bg(termion::color::Red),
                error,
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
            return;
        }

        match state.mode() {
            Mode::Quitting(_) => return,
            Mode::Command => {
                write!(out, "{}:{}", crate::common::goto(1, h), cmdline.cmd()).unwrap()
            }
            Mode::Immediate => write!(
                out,
                "{}{}{}--INSERT--{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                termion::style::Reset,
            )
            .unwrap(),
            Mode::Object => write!(
                out,
                "{}{}{}--SELECT--{}",
                crate::common::goto(1, h),
                termion::style::Bold,
                termion::color::Fg(termion::color::White),
                termion::style::Reset,
            )
            .unwrap(),
        }
    }
}
