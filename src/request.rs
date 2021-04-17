use std::str::FromStr;
use std::fmt;
use std::path::Path;

use crate::{ Result, Error, ErrorKind };
use super::common::*;

pub struct Request {
    pub request_line: HttpRequestLine,
    pub headers: Vec<HttpHeader>,
}

impl Request {
    pub fn from_request_line(s: &str) -> Result<Self> {
        let request_line = HttpRequestLine::from_str(s)
            .map_err(|_| Error(ErrorKind::HttpParseError))?;
        
        Ok(Self {
            request_line,
            headers: Vec::new(),
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