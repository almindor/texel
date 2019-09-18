use crate::components::{Border, Color, Dimension, Position, Selection, Sprite};
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
    fn render_border(&self, out: &mut SyncTerm, p: &Position, d: &Dimension) {
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
        ReadStorage<'a, Color>,
        ReadStorage<'a, Border>,
        ReadStorage<'a, Selection>,
    );

    fn run(&mut self, (mut out, e, p, d, s, c, b, sel): Self::SystemData) {
        for (entity, pos, dim, sprite, color) in (&e, &p, &d, &s, &c).join() {
            write!(out, "{}", color.0).unwrap();

            for t in &sprite.texels {
                write!(
                    out,
                    "{}{}{}",
                    crate::common::goto(pos.x + t.x, pos.y + t.y),
                    color.0,
                    t.symbol
                )
                .unwrap();
            }

            if b.get(entity).is_some() && sel.get(entity).is_some() {
                self.render_border(&mut out, &pos, &dim);
            }
        }

        write!(out, "{}", Color::default().0).unwrap();
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
            Mode::Quitting => return,
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
