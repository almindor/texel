use crate::components::{Border, Color, Dimension, Position, Sprite};
use crate::resources::{CmdLine, Mode, State, SyncTerm};
use specs::{ReadStorage, System};
use std::io::Write;

pub struct ClearScreen;

pub struct SpriteRenderer;

pub struct BorderRenderer;

pub struct CmdLineRenderer;

impl<'a> System<'a> for ClearScreen {
    type SystemData = specs::Write<'a, SyncTerm>;

    fn run(&mut self, mut out: Self::SystemData) {
        write!(out, "{}", termion::clear::All).unwrap();
    }
}

impl<'a> System<'a> for SpriteRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, (mut out, p, s, c): Self::SystemData) {
        use specs::Join;

        for (pos, sprite, color) in (&p, &s, &c).join() {
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
        }

        write!(out, "{}", Color::default().0).unwrap();
    }
}

impl<'a> System<'a> for BorderRenderer {
    type SystemData = (
        specs::Write<'a, SyncTerm>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Dimension>,
        ReadStorage<'a, Border>,
    );

    fn run(&mut self, (mut out, p, d, b): Self::SystemData) {
        use specs::Join;

        for (posision, dimension, _border) in (&p, &d, &b).join() {
            let min_x = posision.x - 1;
            let min_y = posision.y - 1;
            let b_w = i32::from(dimension.w + 1);
            let b_h = i32::from(dimension.h + 1);

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

        if let Some(error) = state.error {
            write!(
                out,
                "{}{}ERR: {}{}",
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
