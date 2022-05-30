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
    fn init_client<U: AsRef<str>>(url: U) -> Result<Client> {
        let mut info: redis::ConnectionInfo =
            url.as_ref().parse().context("failed to parse redis url")?;
        if let Tcp(ref ip, ref port) = &info.addr {
            // url parse returns ipv6 surrounded by []
            // net's TcpStream (used by the client) doesn't work with it surrounded
            info.addr = Tcp(ip.trim_matches(&['[', ']'] as &[char]).to_string(), *port);
        }
        Client::open(info).context("failed to create redis connection")
    }
    pub fn new<U: AsRef<str>, C: Into<String>>(url: U, channel: C) -> Result<Redis> {
        let client = Self::init_client(url)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp() {
        let protocols = ["redis"];
        let auths = ["", "user:password@"];
        let ports = [":6379", ""];
        let hosts = ["127.0.0.1", "[::1]", "hamada.com"];
        for prot in protocols {
            for auth in auths {
                for port in ports {
                    for host in hosts {
                        let conn_string = format!("{}://{}{}{}", prot, auth, host, port);
                        let client = Redis::init_client(conn_string.clone())
                            .expect(format!("{} failed", conn_string).as_str());
                        let conn = client.get_connection_info();
                        let stripped_host = host.trim_matches(&['[', ']'] as &[char]);
                        let port = if port == "" { ":6379" } else { port };
                        assert!(conn.addr.to_string() == format!("{}{}", stripped_host, port));
                        if auth != "" {
                            assert!(conn.redis.username == Some("user".to_string()));
                            assert!(conn.redis.password == Some("password".to_string()));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn test_unix() {
        let client = Redis::init_client("unix:///var/run/redis.sock").expect("unix");
        let conn = client.get_connection_info();
        assert!(conn.addr.to_string() == "/var/run/redis.sock");
    }
}
