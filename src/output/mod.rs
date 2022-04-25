use anyhow::{Context, Result};
use std::io::Write;
use url::Url;

mod redis;
pub use self::redis::Redis;

pub struct Console {}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

pub fn output<S: AsRef<str>>(url: S) -> Result<Box<dyn Write>> {
    let mut u = Url::parse(url.as_ref()).context("failed to parse output url")?;

    match u.scheme() {
        "console" => Ok(Box::new(Console {})),
        "redis" => {
            let channel: String = u.path().trim_start_matches('/').into();
            if channel.is_empty() {
                bail!("channel is missing");
            }
            u.set_path("");
            Ok(Box::new(Redis::new(u, channel)?))
        }
        _ => bail!("unknown output type"),
    }
}
