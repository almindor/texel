use std::io::Write;
use std::vec::Vec;

#[derive(Default)]
pub struct SyncTerm {
    buf: Vec<u8>,
    pub w: u16,
    pub h: u16,
}

impl SyncTerm {
    pub fn new(w: u16, h: u16) -> Self {
        SyncTerm {
            buf: Vec::new(),
            w,
            h,
        }
    }

    pub fn flush_into(&self, out: &mut Write) -> Result<(), std::io::Error> {
        out.write_all(&self.buf)
    }
}

impl Write for SyncTerm {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        // ignored
        Ok(())
    }
}
