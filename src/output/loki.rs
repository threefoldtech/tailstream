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
    username: Option<String>,
    password: Option<String>,
    tenant_id: Option<String>,
    buffer: Vec<u8>,
}

impl Loki {
    pub fn new<U: AsRef<str>>(url: U) -> Result<Self> {
        let parsed = url::Url::parse(url.as_ref())?;
        let username = if parsed.username().is_empty() {
            None
        } else {
            Some(parsed.username().to_string())
        };

        let password = parsed.password().map(|p| p.to_string());

        let tenant_id = parsed
            .query_pairs()
            .find(|(key, _)| key == "tenant_id" || key == "tenant")
            .map(|(_, value)| value.to_string());

        let base_url = format!(
            "{}://{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or("localhost"),
            parsed.port().map(|p| format!(":{}", p)).unwrap_or_default()
        );

        let push_url = format!("{}{}", base_url.trim_end_matches('/'), LOKI_PUSH_PATH);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(Loki {
            client,
            url: push_url,
            username,
            password,
            tenant_id,
            buffer: Vec::new(),
        })
    }

    fn send_line(&mut self, log_line: &str) -> std::io::Result<()> {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        let mut labels = HashMap::new();
        labels.insert("job".to_string(), "tailstream".to_string());

        let payload = LokiPayload {
            streams: vec![LokiStream {
                stream: labels,
                values: vec![[timestamp_ns, log_line.to_string()]],
            }],
        };

        let mut request = self.client.post(&self.url).json(&payload);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        if let Some(tenant_id) = &self.tenant_id {
            request = request.header("X-Scope-OrgID", tenant_id);
        }

        let response = request
            .send()
            .map_err(|e| Error::new(ErrorKind::Other, e))?;

        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("loki push failed: {}", response.status()),
            ));
        }

        Ok(())
    }
}

impl Write for Loki {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let log_line = std::str::from_utf8(&line_bytes)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
                .trim();

            if !log_line.is_empty() {
                self.send_line(log_line)?;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Send any remaining data in buffer
        if !self.buffer.is_empty() {
            let log_line = std::str::from_utf8(&self.buffer)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
                .trim()
                .to_string();

            self.buffer.clear();

            if !log_line.is_empty() {
                self.send_line(&log_line)?;
            }
        }
        Ok(())
    }
}
