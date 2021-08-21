use std::collections::HashMap;
use std::fmt;

use async_std::fs;
use async_std::io;
use async_std::io::prelude::*;
use futures::AsyncWrite;

use super::common::{HttpStatusCode, HttpVersion};
use crate::Result;

pub struct Response {
    pub status_line: HttpStatusLine,
    // TODO: Store &str instead?
    headers: HashMap<&'static str, Vec<String>>,
}

impl Response {
    pub fn new(status_code: u16) -> Self {
        let http_version = HttpVersion(1, 1);
        let status_code = HttpStatusCode(status_code);

        let status_line = HttpStatusLine {
            http_version,
            status_code,
        };

        let mut headers = HashMap::new();
        headers.insert("Server", vec!["polyserve (rust)".to_owned()]);

        Response {
            status_line,
            headers,
        }
    }

    async fn write_head<W>(&self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let status_line = self.status_line.to_string();

        stream.write(status_line.as_bytes()).await?;

        for (field, values) in self.headers.iter() {
            let header = [b"\n", field.as_bytes(), b": ", values.join(",").as_bytes()].concat();

            stream.write(&header).await?;
        }

        stream.write(b"\n\n").await?;

        Ok(())
    }

    pub fn set_header(&mut self, field: &'static str, values: Vec<String>) {
        self.headers.insert(field, values);
    }

    pub async fn send_empty<W>(mut self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.set_header("Content-Length", vec!["0".to_owned()]);
        self.set_header("Content-Type", vec!["text/plain; charset=UTF-8".to_owned()]);

        self.write_head(stream).await?;

        Ok(())
    }

    pub async fn send_file<W>(mut self, mut file: fs::File, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        self.set_header("Content-Length", vec![file_size.to_string()]);

        self.write_head(stream).await?;

        io::copy(&mut file, stream).await?;

        Ok(())
    }

    pub async fn send_str<W>(mut self, s: &str, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.set_header("Content-Length", vec![s.len().to_string()]);
        self.write_head(stream).await?;

        stream.write(s.as_bytes()).await?;

        Ok(())
    }
}

pub struct HttpStatusLine {
    http_version: HttpVersion,
    status_code: HttpStatusCode,
}

impl fmt::Display for HttpStatusLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.http_version,
            self.status_code.0,
            self.status_code.default_reason_phrase(),
        )
    }
}