use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use std::{fmt::Display, io::Write, str::FromStr};
use url::Url;

mod redis;
pub use self::redis::Redis;
mod ws;
pub use ws::WebSocket;

pub fn output<S: AsRef<str>>(url: S, compression: CompressionKind) -> Result<Box<dyn Write>> {
    let mut u = Url::parse(url.as_ref()).context("failed to parse output url")?;

    match u.scheme() {
        // console always ignores compression
        "console" => Ok(Box::new(std::io::stdout())),
        "redis" => {
            let channel: String = u.path().trim_start_matches('/').into();
            if channel.is_empty() {
                bail!("channel is missing");
            }
            u.set_path("");
            let writer = Compression::new(Redis::new(u, channel)?, compression);
            Ok(Box::new(writer))
        }
        "ws" | "wss" => {
            let writer = Compression::new(WebSocket::new(url)?, compression);
            Ok(Box::new(writer))
        }
        _ => bail!("unknown output type"),
    }
}

#[derive(Debug)]
pub enum CompressionKind {
    None,
    Gzip,
}

impl FromStr for CompressionKind {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(CompressionKind::None),
            "gzip" => Ok(CompressionKind::Gzip),
            _ => Err("unknown compression kind"),
        }
    }
}

impl Display for CompressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CompressionKind::None => "none",
                CompressionKind::Gzip => "gzip",
            }
        )
    }
}

struct Compression<W> {
    inner: W,
    kind: CompressionKind,
}

impl<W> Compression<W>
where
    W: Write,
{
    pub fn new(inner: W, kind: CompressionKind) -> Self {
        Compression { inner, kind }
    }
}

impl<W> Write for Compression<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.kind {
            CompressionKind::None => self.inner.write(buf),
            CompressionKind::Gzip => {
                let mut enc = GzEncoder::new(Vec::new(), flate2::Compression::best());
                enc.write_all(buf)?;

                self.inner.write(&enc.finish()?)?;

                // write must return the length of its input or it will panic
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
