use crate::common::{SymbolStyle, Texel};
use crate::resources::ColorPalette;
use big_enum_set::BigEnumSet;
use std::io::Write;
use std::vec::Vec;

#[derive(Debug, Default)]
struct TexelBuf {
    size_x: usize,
    size_y: usize,
    buf: Vec<Texel>,
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
            let (x, y) = self.deindex(i);
            let texel = Texel {
                x,
                y,
                symbol: ' ',
                bg: ColorPalette::default_bg_u8(),
                fg: ColorPalette::default_fg_u8(),
                styles: BigEnumSet::new(),
            };

            self.buf.push(texel);
        }
    }

    pub fn set_texel(&mut self, t: Texel) {
        let index = self.index(t.x, t.y);

        if index < self.buf.len() {
            self.buf[index] = t;
        }
    }

    pub fn set_texels(&mut self, texels: Vec<Texel>) {
        for t in texels {
            self.set_texel(t);
        }
    }

    fn texel_match(&self, texel: &Texel) -> bool {
        self.buf[self.index(texel.x, texel.y)] == *texel
    }

    pub fn diff(newer: &Self, older: &Self) -> Vec<Texel> {
        let mut vec = Vec::with_capacity(newer.buf.capacity());

        for texel in &newer.buf {
            if !older.texel_match(texel) {
                vec.push(*texel);
            }
        }

        vec
    }

    fn index(&self, x: i32, y: i32) -> usize {
        self.size_x * ((y - 1) as usize) + ((x - 1) as usize)
    }

    fn deindex(&self, i: usize) -> (i32, i32) {
        ((i % self.size_x) as i32 + 1, (i / self.size_x) as i32 + 1)
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

    pub fn write_texels(&mut self, texels: Vec<Texel>) {
        let buf = self.buf_mut();

        buf.set_texels(texels);
    }

    pub fn write_line(
        &mut self,
        start_x: i32,
        y: i32,
        source: impl std::fmt::Display,
        bg: u8,
        fg: u8,
        styles: BigEnumSet<SymbolStyle>,
    ) {
        let text = format!("{}", source);
        let buf = self.buf_mut();

        let mut x: i32 = start_x;
        for symbol in text.chars() {
            buf.set_texel(Texel {
                x,
                y,
                symbol,
                fg,
                bg,
                styles: styles,
            });

            x += 1;
        }
    }

    pub fn write_line_default(&mut self, start_x: i32, y: i32, source: impl std::fmt::Display) {
        self.write_line(
            start_x,
            y,
            source,
            ColorPalette::default_bg_u8(),
            ColorPalette::default_fg_u8(),
            BigEnumSet::new(),
        );
    }

    pub fn flip_buffers(&mut self) {
        self.index = 1 - self.index;
        self.buf_mut().clear();
    }

    pub fn flush_into(&self, out: &mut dyn Write) -> Result<(), std::io::Error> {
        let vec = TexelBuf::diff(self.buf(), self.previous_buf());
        
        for texel in vec {
            write!(out, "{}", texel)?;
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
