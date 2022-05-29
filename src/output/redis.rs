use anyhow::{Context, Result};
use r2d2::Pool;
use redis::ConnectionAddr::Tcp;
use redis::{Client, Commands};
use std::io::{Error, ErrorKind, Write};

pub struct Redis {
    pool: Pool<Client>,
    channel: String,
}

impl Redis {
    pub fn new<U: AsRef<str>, C: Into<String>>(url: U, channel: C) -> Result<Redis> {
        let mut info: redis::ConnectionInfo =
            url.as_ref().parse().context("failed to parse redis url")?;
        if let Tcp(ref ip, ref port) = &info.addr {
            // url parse returns ipv6 surrounded by []
            // net's TcpStream (used by the client) doesn't work with it surrounded
            info.addr = Tcp(ip.trim_matches(&['[', ']'] as &[char]).to_string(), *port);
        }
        let client = Client::open(info).context("failed to create redis connection")?;
        let pool = Pool::builder()
            .max_size(1)
            .idle_timeout(Some(std::time::Duration::from_secs(5 * 60)))
            .build(client)
            .context("failed to create redis pool")?;

        Ok(Redis {
            pool,
            channel: channel.into(),
        })
    }
}

impl Write for Redis {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut client = match self.pool.get() {
            Ok(client) => client,
            Err(err) => {
                return Err(Error::new(ErrorKind::ConnectionRefused, err));
            }
        };

        match client.publish(&self.channel, buf) {
            Ok(()) => Ok(buf.len()),
            Err(err) => Err(Error::new(ErrorKind::Other, err)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
