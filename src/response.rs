use std::collections::HashMap;
use std::fmt;

use async_std::fs;
use async_std::io;
use async_std::io::prelude::*;
use futures::{AsyncRead, AsyncWrite};

use super::common::{HttpStatusCode, HttpVersion};
use crate::Result;

pub struct Response {
    pub status_line: HttpStatusLine,
    headers: HashMap<&'static str, Vec<String>>,
}

impl Response {
    fn new(status_code: u16) -> Self {
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

    pub fn empty(status_code: u16) -> Self {
        let mut response = Self::new(status_code);

        response.set_header("Content-Length", vec!["0".to_owned()]);
        response.set_header("Content-Type", vec!["text/plain; charset=UTF-8".to_owned()]);

        response
    }

    pub async fn from_file(status_code: u16, mut file: fs::File) -> Result<Self> {
        let mut response = Self::new(status_code);

        let metadata = file.metadata().await?;

        let file_size = metadata.len();

        response.set_header("Content-Length", vec![file_size.to_string()]);

        //response.body = file;

        Ok(response)
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

    pub async fn end<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.write_head(stream).await?;

        // TODO: If data, write to stream
        //io::copy(file, stream).await?;

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