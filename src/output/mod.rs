use anyhow::Result;
use std::io::Write;

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

pub fn output<S: AsRef<str>>(_url: S) -> Result<Box<dyn Write>> {
    if true {
        Ok(Box::new(Console {}))
    } else {
        Ok(Box::new(Redis::new()?))
    }
}
