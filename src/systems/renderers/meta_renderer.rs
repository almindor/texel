use crate::common::index_from_one;
use crate::components::{Bookmark, Position, Position2D, Selection, Sprite};
use crate::os::Terminal;
use crate::resources::{FrameBuffer, State};
use legion::prelude::*;

const METADATA_WIDTH: i32 = 25;

pub fn render_meta_info(world: &mut World, state: &State, out: &mut FrameBuffer) {
    if !state.show_meta {
        return;
    }

    let ts = Terminal::terminal_size();
    let w = i32::from(ts.0);
    // let h = i32::from(ts.1);

    out.write_line_default(1, 1, "=====BOOKMARKS=====");

    let query = <(Read<Bookmark>, Read<Position2D>)>::query();
    for (i, (bookmark, pos)) in query.iter(world).enumerate() {
        out.write_line_default(1, (i + 2) as i32, index_from_one(bookmark.0));
    }

    let x = w - METADATA_WIDTH;
    let mut y = 0i32;
    let query = <(Read<Sprite>, Read<Position>)>::query().filter(component::<Selection>());
    for (i, (sprite, pos)) in query.iter(world).enumerate() {
        match sprite.id {
            Some(id) => out.write_line_default(x, y, format!("===  {}  ===", id)),
            None => out.write_line_default(x, y, "===<NONE>==="),
        }

        y += 1;
        out.write_line_default(x, y, format!(" Labels: {}", sprite.labels.join(",")));
        y += 1;
    }
}
