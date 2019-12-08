use std::io::Write;
use std::vec::Vec;
use texel_types::{SymbolStyles, Texel, Texels, Position2D};

#[derive(Debug, Default)]
struct TexelBuf {
    size_x: usize,
    size_y: usize,
    buf: Texels,
}

impl TexelBuf {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        let mut result = Self {
            size_x,
            size_y,
            buf: Vec::with_capacity(size_x * size_y),
        };
        result.clear();

        result
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        for i in 0..self.buf.capacity() {
            let pos = self.deindex(i);
            let texel = Texel {
                pos,
                symbol: ' ',
                bg: texel_types::DEFAULT_BG_U8,
                fg: texel_types::DEFAULT_FG_U8,
                styles: SymbolStyles::new(),
            };

            self.buf.push(texel);
        }
    }

    pub fn set_texel(&mut self, texel: Texel) {
        let index = self.index(texel.pos);

        if index < self.buf.len() {
            self.buf[index] = texel;
        }
    }

    pub fn set_texels(&mut self, texels: Texels) {
        for t in texels {
            self.set_texel(t);
        }
    }

    pub fn override_texel_bg(&mut self, texel: Texel) {
        let index = self.index(texel.pos);

        if index >= self.buf.len() {
            return;
        }

        if let Some(existing) = self.buf.get_mut(index) {
            existing.bg = texel.bg;
        } else {
            self.set_texel(texel);
        }
    }

    fn texel_match(&self, texel: &Texel) -> bool {
        self.buf[self.index(texel.pos)] == *texel
    }

    pub fn diff(newer: &Self, older: &Self) -> Texels {
        let mut vec = Vec::with_capacity(newer.buf.capacity());

        for texel in &newer.buf {
            if !older.texel_match(texel) {
                vec.push(texel.clone());
            }
        }

        vec
    }

    fn index(&self, pos: Position2D) -> usize {
        self.size_x * ((pos.y - 1) as usize) + ((pos.x - 1) as usize)
    }

    fn deindex(&self, i: usize) -> Position2D {
        Position2D {
            x: (i % self.size_x) as i32 + 1,
            y: (i / self.size_x) as i32 + 1,
        }
    }
}

#[derive(Default)]
pub struct SyncTerm {
    buffers: [TexelBuf; 2],
    index: usize,
    cursor_x: i32,
    cursor_y: i32,
}

impl SyncTerm {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        SyncTerm {
            buffers: [TexelBuf::new(size_x, size_y), TexelBuf::new(size_x, size_y)],
            index: 0,
            cursor_x: 1,
            cursor_y: 1,
        }
    }

    pub fn set_cursor_pos(&mut self, cursor_x: i32, cursor_y: i32) {
        self.cursor_x = cursor_x;
        self.cursor_y = cursor_y;
    }

    pub fn write_texel(&mut self, texel: Texel) {
        let buf = self.buf_mut();

        buf.set_texel(texel);
    }

    pub fn write_texels(&mut self, texels: Texels) {
        let buf = self.buf_mut();

        buf.set_texels(texels);
    }

    pub fn override_texel_bg(&mut self, texel: Texel) {
        let buf = self.buf_mut();

        buf.override_texel_bg(texel);
    }

    pub fn write_line(
        &mut self,
        start_x: i32,
        y: i32,
        source: impl std::fmt::Display,
        bg: u8,
        fg: u8,
        styles: SymbolStyles,
    ) {
        let text = format!("{}", source);
        let buf = self.buf_mut();

        let mut x: i32 = start_x;
        for symbol in text.chars() {
            buf.set_texel(Texel {
                pos: Position2D { x, y },
                symbol,
                fg,
                bg,
                styles,
            });

            x += 1;
        }
    }

    pub fn write_line_default(&mut self, start_x: i32, y: i32, source: impl std::fmt::Display) {
        self.write_line(
            start_x,
            y,
            source,
            texel_types::DEFAULT_BG_U8,
            texel_types::DEFAULT_FG_U8,
            SymbolStyles::new(),
        );
    }

    pub fn flip_buffers(&mut self) {
        self.index = 1 - self.index;
        self.buf_mut().clear();
    }

    pub fn flush_into(&self, out: &mut dyn Write) -> Result<(), std::io::Error> {
        use crate::common::texel_to_string;
        let vec = TexelBuf::diff(self.buf(), self.previous_buf());

        for texel in vec {
            write!(out, "{}", texel_to_string(&texel))?;
        }

        out.flush()?;
        write!(out, "{}", crate::common::goto(self.cursor_x, self.cursor_y))
    }

    fn buf_mut(&mut self) -> &mut TexelBuf {
        &mut self.buffers[self.index]
    }

    fn buf(&self) -> &TexelBuf {
        &self.buffers[self.index]
    }

    fn previous_buf(&self) -> &TexelBuf {
        &self.buffers[1 - self.index]
    }
}
