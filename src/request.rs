use std::str::FromStr;
use std::fmt;
use std::path::{ PathBuf, Path };

use futures::io::BufReader;
use async_std::io::prelude::*;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;

use crate::{ Result, Error, ErrorKind };
use super::common::*;

pub struct Request {
    pub request_line: HttpRequestLine,
    pub headers: Vec<HttpHeader>,
}

impl Request {
    pub async fn from_stream(stream: &TcpStream) -> Result<Self> {
        let reader = BufReader::with_capacity(1024, stream);
        let mut lines = reader.lines();

        let request_line = match lines.next().await {
            Some(line) => Ok(line),
            None => Err(Error(ErrorKind::HttpParse)),
        }?
        .unwrap();

        let request_line = HttpRequestLine::from_str(request_line.as_ref())?;

        let mut headers = Vec::new();

        while let Some(Ok(line)) = lines.next().await {
            if line.is_empty() {
                // End of request header
                break;
            }

            if let Ok(header) = HttpHeader::from_str(line.as_ref()) {
                headers.push(header);
            }
        };

        // TODO: Content-Length, Transfer-Encoding, body

        Ok(Request {
            request_line,
            headers,
        })
    }

    pub fn method(&self) -> &HttpMethod {
        &self.request_line.method
    }

    pub fn path(&self) -> &Path {
        self.request_line.path.as_path()
    }

    pub fn http_version(&self) -> &HttpVersion {
        &self.request_line.http_version
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.request_line)?;
        
        for header in &self.headers {
            write!(f, "{}\n", header)?;
        }

        write!(f, "\n")
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.request_line)
    }
}

pub struct HttpRequestLine {
    pub method: HttpMethod,
    pub path: PathBuf,
    pub http_version: HttpVersion,
}

impl FromStr for HttpRequestLine {
    type Err = Error;

    fn from_str(s: &str) -> Result<HttpRequestLine> {
        let mut parts = s.split_whitespace();

        let method = parts.next();
        let path = parts.next();
        let http_version = parts.next();

        match (method, path, http_version) {
            (Some(method), Some(path), Some(http_version)) => {
                Ok(HttpRequestLine {
                    method: HttpMethod::from_str(method).unwrap(),
                    path: PathBuf::from(path),
                    http_version: HttpVersion::from_str(http_version).unwrap(),
                })
            },
            _ => Err(Error(ErrorKind::HttpParse)),
        }
    }
}

impl fmt::Display for HttpRequestLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.method, self.path.display(), self.http_version)
    }
}