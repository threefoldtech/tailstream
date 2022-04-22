use std::io::Write;

use anyhow::Result;

mod redis;
pub use redis::Redis;

pub struct Console {}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}
