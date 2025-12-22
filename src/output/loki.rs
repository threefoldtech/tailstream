use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Write};
use std::time::Duration;

const LOKI_PUSH_PATH: &str = "/loki/api/v1/push";

#[derive(Serialize)]
struct LokiStream {
    stream: HashMap<String, String>,
    values: Vec<[String; 2]>,
}

#[derive(Serialize)]
struct LokiPayload {
    streams: Vec<LokiStream>,
}

pub struct Loki {
    client: reqwest::blocking::Client,
    url: String,
}

impl Loki {
    pub fn new<U: AsRef<str>>(url: U) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let base_url = url.as_ref().trim_end_matches('/');
        let push_url = format!("{}{}", base_url, LOKI_PUSH_PATH);

        Ok(Loki {
            client,
            url: push_url,
        })
    }
}

impl Write for Loki {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        let log_line = std::str::from_utf8(buf)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .trim()
            .to_string();

        let mut labels = HashMap::new();
        labels.insert("job".to_string(), "tailstream".to_string());

        let payload = LokiPayload {
            streams: vec![LokiStream {
                stream: labels,
                values: vec![[timestamp_ns, log_line]],
            }],
        };

        let response = self
            .client
            .post(&self.url)
            .json(&payload)
            .send()
            .map_err(|e| Error::new(ErrorKind::Other, e))?;

        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("loki push failed: {}", response.status()),
            ));
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
