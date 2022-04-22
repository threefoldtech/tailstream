use anyhow::Result;
use std::io::Write;

pub struct Redis {}

impl Redis {
    pub fn new() -> Result<Redis> {
        Ok(Redis {})
    }
}

impl Write for Redis {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}
