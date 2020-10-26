use crate::common::{index_from_one, shortened_str};
use crate::components::{Bookmark, Selection, Sprite};
use crate::os::Terminal;
use crate::resources::{FrameBuffer, State};
use legion::*;

const METADATA_WIDTH: i32 = 25;

pub fn render_meta_info(world: &mut World, state: &State, out: &mut FrameBuffer) {
    if !state.show_meta {
        return;
    }

    let ts = Terminal::terminal_size();
    let w = i32::from(ts.0);

    let filename = state.filename();
    out.write_line_default(w / 2 - filename.len() as i32 / 2, 1, format!("<{}>", filename));
    out.write_line_default(1, 1, "=====BOOKMARKS=====");

    let mut query = <Read<Bookmark>>::query();
    for (i, bookmark) in query.iter(world).enumerate() {
        out.write_line_default(1, (i + 2) as i32, index_from_one(bookmark.0));
    }

    let x = w - METADATA_WIDTH;
    let mut y = 0i32;
    let mut query = <Read<Sprite>>::query().filter(component::<Selection>());
    for sprite in query.iter(world) {
        match sprite.id {
            Some(id) => out.write_line_default(x, y, format!("===  {}  ===", id)),
            None => out.write_line_default(x, y, "===<NONE>==="),
        }

        y += 1;
        let labels = sprite
            .labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join(",");
        let label_str = shortened_str(&labels, 15);
        if label_str.1 {
            out.write_line_default(x, y, format!(" Labels: {}...", label_str.0));
        } else {
            out.write_line_default(x, y, format!(" Labels: {}", label_str.0));
        }

        y += 1;
    }
}
