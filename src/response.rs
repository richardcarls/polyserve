use std::fmt;
use std::str::FromStr;

use async_std::fs;
use async_std::io;
use async_std::io::prelude::*;
use futures::AsyncWrite;

use crate::common::*;
use crate::Result;

pub struct Response {
    pub status_line: HttpStatusLine,
    pub headers: Vec<HttpHeader>,
}

impl Response {
    pub fn new(status_code: u16) -> Self {
        let http_version = HttpVersion(1, 1);
        let status_code = HttpStatusCode(status_code);

        let status_line = HttpStatusLine {
            http_version,
            status_code,
        };

        let mut headers = Vec::new();
        headers.push(HttpHeader::new("Server", &["polyserve (rust)"]));

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

        for header in self.headers.iter() {
            let header = [b"\n", header.name.as_bytes(), b": ", header.values.join(",").as_bytes()].concat();

            stream.write(&header).await?;
        }

        stream.write(b"\n\n").await?;

        Ok(())
    }

    pub fn set_header(&mut self, name: &str, values: &[&str]) {
        let header = HttpHeader::new(name, values);

        self.headers.push(header);
    }

    pub async fn send_empty<W>(mut self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.set_header("Content-Length", &["0"]);
        self.set_header("Content-Type", &["text/plain; charset=UTF-8"]);

        self.write_head(stream).await?;

        Ok(())
    }

    pub async fn send_file<W>(mut self, mut file: fs::File, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        self.set_header("Content-Length", &[file_size.to_string().as_str()]);

        self.write_head(stream).await?;

        io::copy(&mut file, stream).await?;

        Ok(())
    }

    pub async fn send_str<W>(mut self, s: &str, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.set_header("Content-Length", &[s.len().to_string().as_str()]);
        self.write_head(stream).await?;

        stream.write(s.as_bytes()).await?;

        Ok(())
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new(404)
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.status_line)?;
        for header in &self.headers {
            write!(f, "{}\n", header)?;
        }

        write!(f, "\n")
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status_line)
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
