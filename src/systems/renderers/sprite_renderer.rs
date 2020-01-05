use crate::common::{scene_for_help_index, Mode, Scene, SelectedInfo};
use crate::components::{Border, Dimension, Position, Position2D, Selection, Sprite};
use crate::os::Terminal;
use crate::resources::{FrameBuffer, State};
use specs::{Entities, Join, Read, ReadStorage, System};
use texel_types::{SymbolStyles, Texel, DEFAULT_BG_U8, DEFAULT_FG_U8};

pub struct SpriteRenderer;

impl<'a> System<'a> for SpriteRenderer {
    type SystemData = (
        specs::Write<'a, FrameBuffer>,
        Entities<'a>,
        Read<'a, State>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Dimension>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, Border>,
        ReadStorage<'a, Selection>,
    );

    fn run(&mut self, (mut out, e, state, p, d, s, b, sel): Self::SystemData) {
        if let Mode::Help(index) = state.mode() {
            render_scene(&mut out, &state, scene_for_help_index(index));
            return; // show help, done
        }

        let mut selected_info = SelectedInfo::from(state.offset);

        // TODO: optimize using FlaggedStorage
        let mut sorted = (&e, &p, &d, &s).join().collect::<Vec<_>>();
        sorted.sort_by(|&a, &b| b.1.z.cmp(&a.1.z));

        for (entity, pos, dim, sprite) in sorted {
            render_sprite(&mut out, &state, &pos, &sprite);

            if b.contains(entity) && sel.contains(entity) {
                render_border(&mut out, &state, &pos, *dim);
                selected_info.append(sprite, pos);
            }
        }

        // location info status line
        let ts = Terminal::terminal_size();
        let texels = selected_info.texels(&state, ts.0, ts.1);

        out.write_texels(texels);
    }
}

fn render_scene(out: &mut FrameBuffer, state: &State, scene: Scene) {
    for obj in scene.current().objects {
        render_sprite(out, state, &obj.1, &obj.0);
    }
}

fn render_sprite(out: &mut FrameBuffer, state: &State, p: &Position, s: &Sprite) {
    for t in s.frame_iter() {
        print_texel(out, state, p, t);
    }
}

fn print_texel(out: &mut FrameBuffer, state: &State, p: &Position, t: &Texel) {
    let pos2d: Position2D = (*p + t.pos).into();
    let abs_texel = Texel {
        pos: pos2d - state.offset,
        symbol: t.symbol,
        bg: t.bg,
        fg: t.fg,
        styles: t.styles,
    };

    out.write_texel(abs_texel);
}

fn render_border(out: &mut FrameBuffer, state: &State, p: &Position, d: Dimension) {
    let min_x = std::cmp::max(0, p.x - 1);
    let min_y = std::cmp::max(0, p.y - 1);
    let b_w = i32::from(d.w + 1);
    let b_h = i32::from(d.h + 1);

    let t_side = Texel {
        symbol: '|',
        pos: Position2D::default(),
        styles: SymbolStyles::new(),
        bg: DEFAULT_BG_U8,
        fg: DEFAULT_FG_U8,
    };

    let t_top = Texel {
        symbol: '-',
        pos: Position2D::default(),
        styles: SymbolStyles::new(),
        bg: DEFAULT_BG_U8,
        fg: DEFAULT_FG_U8,
    };

    let t_bottom = Texel {
        symbol: '_',
        pos: Position2D::default(),
        styles: SymbolStyles::new(),
        bg: DEFAULT_BG_U8,
        fg: DEFAULT_FG_U8,
    };

    for y in min_y..=min_y + b_h {
        let pos_left = Position { x: min_x, y, z: p.z };
        let pos_right = Position {
            x: min_x + b_w,
            y,
            z: p.z,
        };

        print_texel(out, state, &pos_left, &t_side);
        print_texel(out, state, &pos_right, &t_side);
    }

    for x in min_x..=min_x + b_w {
        let pos_top = Position { x, y: min_y, z: p.z };
        let pos_bottom = Position {
            x,
            y: min_y + b_h,
            z: p.z,
        };

        print_texel(out, state, &pos_top, &t_top);
        print_texel(out, state, &pos_bottom, &t_bottom);
    }
}
