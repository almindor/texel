use crate::os::Terminal;
use crate::resources::State;
use texel_types::{ColorMode, Position, Position2D, Sprite, SymbolStyle, SymbolStyles, Texels};

// meta info for showing in the bottom right corner
// use for multiples too
#[derive(Debug, Clone, Copy)]
pub struct SelectedInfo {
    pub selected_count: usize,
    pub frame_index: usize,
    pub frame_count: usize,
    pub pos: Position,      // top left position
    pub offset: Position2D, // viewport offset
}

impl Default for SelectedInfo {
    fn default() -> Self {
        SelectedInfo {
            selected_count: 0,
            frame_index: 0,
            frame_count: 0,
            pos: Position {
                x: i32::max_value(),
                y: i32::max_value(),
                z: 0,
            },
            offset: Position2D::default(),
        }
    }
}

impl From<Position2D> for SelectedInfo {
    fn from(pos: Position2D) -> Self {
        let mut result = SelectedInfo::default();
        result.offset = pos;

        result
    }
}

/*
    // color selection
    let sc = (state.color(ColorMode::Bg), state.color(ColorMode::Fg));
    let saved_symbol = if state.unsaved_changes() { "*" } else { " " };

    out.write_line_default(w - 35, h - 1, saved_symbol);
    out.write_line(w - 34, h - 1, "▞", sc.0, sc.1, SymbolStyles::new());
    out.set_cursor_pos(w - 1, h - 1);
*/

pub const SELECTED_INFO_TEMPLATE: &str = "*▞     ,   ,    :    /    :    ,    ";

impl SelectedInfo {
    pub fn append(&mut self, sprite: &Sprite, pos: &Position) {
        self.selected_count += 1;
        self.frame_index = sprite.frame_index();
        self.frame_count = sprite.frame_count();

        if pos.x < self.pos.x {
            self.pos.x = pos.x;
        }

        if pos.y < self.pos.y {
            self.pos.y = pos.y;
        }

        self.pos.z = pos.z;
        if self.selected_count > 1 {
            self.pos.z = 0; // don't show
        }
    }

    pub fn texels(&self, state: &State, w: u16, h: u16) -> Texels {
        // TODO: optimize styles/colors in template into singleton
        let start = Position2D {
            x: i32::from(w) - (SELECTED_INFO_TEMPLATE.len() as i32) - 1,
            y: i32::from(h) - 1,
        };
        let mut line = texel_types::texels_from_str(SELECTED_INFO_TEMPLATE, start);
        let bold = SymbolStyles::only(SymbolStyle::Bold);

        if self.selected_count > 0 {
            write_coords_to_line(self.pos.x, self.pos.y, Some(self.pos.z), 7, &mut line);
            write_coords_to_line(
                self.frame_index as i32 + 1,
                self.frame_count as i32,
                None,
                21,
                &mut line,
            );
        }
        write_coords_to_line(self.offset.x, self.offset.y, None, 31, &mut line);

        let white = Terminal::grayscale_u8(23);
        let dark_green = Terminal::rgb_u8(1, 3, 1);
        let dark_blue = Terminal::rgb_u8(1, 1, 3);
        line[1].fg = state.color(ColorMode::Fg);
        line[1].bg = state.color(ColorMode::Bg);
        line[0].styles = SymbolStyles::only(SymbolStyle::Bold);
        line[0].symbol = if state.unsaved_changes() { '*' } else { ' ' };

        for t in line.iter_mut().skip(3).take(13) {
            t.styles = bold; // selected position in bold
            t.fg = white;
        }

        for t in line.iter_mut().skip(17).take(9) {
            t.fg = dark_green;
        }

        for t in line.iter_mut().skip(27) {
            t.fg = dark_blue;
        }

        line
    }
}

fn write_coords_to_line(x: i32, y: i32, maybe_z: Option<i32>, start_x: usize, line: &mut Texels) -> usize {
    let str_x = format!("{}", x);
    let str_y = format!("{}", y);

    texel_types::write_to_texels(&str_x, line, start_x - str_x.len());
    texel_types::write_to_texels(&str_y, line, start_x + 4 - str_y.len());

    if let Some(z) = maybe_z {
        let str_z = format!("{}", z);
        texel_types::write_to_texels(&str_z, line, start_x + 8 - str_z.len());
        8
    } else {
        4
    }
}
