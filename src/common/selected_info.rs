use crate::os::Terminal;
use texel_types::{Position, Position2D, Sprite, SymbolStyle, SymbolStyles, Texels};

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

const SELECTED_INFO_TEMPLATE: &str = "{   ,   }:(   ,   ):[   /   ]";

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

    pub fn texels(&self, w: u16, h: u16) -> Texels {
        // TODO: optimize styles/colors in template into singleton
        let start = Position2D {
            x: i32::from(w) - (SELECTED_INFO_TEMPLATE.len() as i32) - 1,
            y: i32::from(h) - 1,
        };
        let mut line = texel_types::texels_from_str(SELECTED_INFO_TEMPLATE, start);
        let bold = SymbolStyles::only(SymbolStyle::Bold);

        let mut write_coords_to_line = |x, y, start_x| {
            let str_x = format!("{}", x);
            let str_y = format!("{}", y);
            texel_types::write_to_texels(&str_x, &mut line, start_x - str_x.len());
            texel_types::write_to_texels(&str_y, &mut line, start_x + 4 - str_y.len());
        };

        write_coords_to_line(self.offset.x, self.offset.y, 4);

        if self.selected_count > 0 {
            write_coords_to_line(self.pos.x, self.pos.y, 14);
        }

        if self.frame_count > 1 {
            write_coords_to_line(self.frame_index as i32, self.frame_count as i32, 24);
        }

        let white = Terminal::grayscale_u8(23);
        let dark_green = Terminal::rgb_u8(1, 3, 1);
        let dark_blue = Terminal::rgb_u8(1, 1, 3);

        for i in 0..9 {
            line[i].fg = dark_blue;
        }

        for i in 10..19 {
            line[i].styles = bold; // selected position in bold
            line[i].fg = white;
        }

        for i in 20..line.len() {
            line[i].fg = dark_green;
        }

        line
    }
}
