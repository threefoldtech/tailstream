use anyhow::{Context, Result};
use r2d2::ManageConnection;
use redis::{cmd, Client};
use std::io::{Error, ErrorKind, Write};

pub struct Redis {
    client: Client,
    channel: String,
}

impl Redis {
    pub fn new<U: AsRef<str>, C: Into<String>>(url: U, channel: C) -> Result<Redis> {
        //let info: redis::ConnectionInfo = url.as_ref().parse()?;

        let client = Client::open(url.as_ref()).context("failed to create redis connection")?;

        Ok(Redis {
            client,
            channel: channel.into(),
        })
    }
}

impl Write for Redis {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut client = match self.client.connect() {
            Ok(client) => client,
            Err(err) => {
                return Err(Error::new(ErrorKind::ConnectionRefused, err));
            }
        };

        match cmd("PUBLISH")
            .arg(&self.channel)
            .arg(buf)
            .query(&mut client)
        {
            Ok(()) => Ok(buf.len()),
            Err(err) => Err(Error::new(ErrorKind::Other, err)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
