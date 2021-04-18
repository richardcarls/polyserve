use std::fmt;
use std::collections::HashMap;

use async_std::io::prelude::*;
use async_std::net::TcpStream;

use crate::{ Result, Error, ErrorKind };
use super::common::{ HttpVersion, HttpStatusCode };

pub struct Response {
    pub status_line: HttpStatusLine,
    headers: HashMap<&'static str, Vec<String>>
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

    pub fn set_header(&mut self, field: &'static str, values: Vec<String>) {
        self.headers.insert(field, values);
    }

    pub async fn write_head(&self, stream: &mut TcpStream) -> Result<()> {
        let status_line = self.status_line.to_string();

        stream.write(&[status_line.as_bytes(), b"\n"].concat()).await
            .map_err(|err| Error(ErrorKind::HttpParseError))?;

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