use crate::common::{scene_for_help_index, Scene, SelectedInfo};
use crate::components::{Dimension, Position, Position2D, Selection, Sprite};
use crate::os::Terminal;
use crate::resources::{FrameBuffer, State};
use legion::prelude::*;
use texel_types::{SymbolStyles, Texel, DEFAULT_BG_U8, DEFAULT_FG_U8};

pub fn render_sprites(world: &mut World, state: &mut State, out: &mut FrameBuffer) {
    if let Some(index) = state.show_help() {
        render_scene(out, &state, scene_for_help_index(index));
        return; // show help, done
    }

    let mut selected_info = SelectedInfo::from(state.offset);

    let query = <(Read<Position>, Read<Dimension>, Read<Sprite>, TryRead<Selection>)>::query();

    // TODO: optimize
    let mut sorted = Vec::new();

    for tuple in query.iter(world) {
        sorted.push(tuple);
    }
    sorted.sort_by(|a, b| b.0.z.cmp(&a.0.z));

    for (pos, dim, sprite, is_selected) in sorted {
        render_sprite(out, state, &pos, &sprite);

        if is_selected.is_some() {
            render_border(out, &state, &pos, *dim);
            selected_info.append(&sprite, &pos);
        }
    }

    // location info status line
    let ts = Terminal::terminal_size();
    let texels = selected_info.texels(&state, ts.0, ts.1);

    out.write_texels(texels);
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
