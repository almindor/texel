use std::io::Write;
use std::vec::Vec;

#[derive(Default)]
pub struct SyncTerm {
    buf: Vec<u8>,
}

impl SyncTerm {
    pub fn new() -> Self {
        SyncTerm {
            buf: Vec::with_capacity(32768),
        }
    }

    pub fn flush_into(&self, out: &mut dyn Write) -> Result<(), std::io::Error> {
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
