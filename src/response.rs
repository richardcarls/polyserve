use std::fmt;

use async_std::io::prelude::*;
use async_std::net::TcpStream;

use crate::Result;
use super::common::{ HttpVersion, HttpStatusCode };

pub struct Response {
    pub status_line: HttpStatusLine,
}

impl Response {
    pub fn new(status_code: u16, reason_phrase: Option<&str>) -> Self {
        let http_version = HttpVersion(1, 1);
        let status_code = HttpStatusCode(status_code);
        let reason_phrase = reason_phrase
            .unwrap_or(status_code.default_reason_phrase())
            .to_owned();

        let status_line = HttpStatusLine {
            http_version,
            status_code,
            reason_phrase,
        };

        Response {
            status_line,
        }
    }

    pub async fn write_head(&self, stream: &mut TcpStream) -> Result<()> {
        let status_line = self.status_line.to_string();

        stream.write(status_line.as_bytes()).await.unwrap_or_default();

        Ok(())
    }
}

pub struct HttpStatusLine {
    http_version: HttpVersion,
    status_code: HttpStatusCode,
    reason_phrase: String,
}

impl fmt::Display for HttpStatusLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.http_version,
            self.status_code.0,
            self.reason_phrase,
        )
    }
}