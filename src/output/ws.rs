use anyhow::Result;
use r2d2::{ManageConnection, Pool};
use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::{fmt::Display, io::Write};
use websocket::{ClientBuilder, Message, WebSocketError};

const PING: [u8; 0] = [];

pub struct WebSocket {
    pool: Pool<WebsocketManager>,
}

impl WebSocket {
    pub fn new<U: AsRef<str>>(u: U) -> Result<Self> {
        let mgr = WebsocketManager {
            url: u.as_ref().into(),
        };

        let pool = Pool::builder()
            .max_size(1)
            .idle_timeout(Some(Duration::from_secs(20 * 60)))
            .build(mgr)?;

        Ok(WebSocket { pool })
    }
}

impl Write for WebSocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| Error::new(ErrorKind::ConnectionRefused, e))?;

        let msg = Message::binary(buf);
        conn.send_message(&msg)
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
enum PoolError {
    ParseError,
    SocketError(WebSocketError),
}

impl std::error::Error for PoolError {}

impl Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PoolError::ParseError => write!(f, "invalid websocket url"),
            PoolError::SocketError(err) => write!(f, "{}", err),
        }
    }
}
struct WebsocketManager {
    url: String,
}

impl ManageConnection for WebsocketManager {
    type Connection = websocket::client::sync::Client<
        Box<dyn websocket::stream::sync::NetworkStream + std::marker::Send>,
    >;
    type Error = PoolError;

    /// Attempts to create a new connection.
    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut builder = ClientBuilder::new(&self.url).map_err(|_| PoolError::ParseError)?;
        builder.connect(None).map_err(PoolError::SocketError)
    }

    // is_valid is called before each connection borrow from the pool. Which causes
    // a ping call to be made before sending the actual data.
    // todo: find another way to check if connection is still on.
    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.send_message(&Message::ping(&PING[..]))
            .map_err(PoolError::SocketError)
    }

    // has_broken is called before connection is returned to pool
    // need to be non-blocking. dummy implementation is to return false
    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}
