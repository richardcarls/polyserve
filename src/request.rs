use std::path::PathBuf;
use std::fmt;

use futures::io::BufReader;
use async_std::net::TcpStream;

use super::common::*;

pub struct Request {
    pub method: HttpMethod,
    pub path: PathBuf,
    pub http_version: HttpVersion,
    pub headers: Vec<HttpHeader>,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}\n", self.method, self.path.display(), self.http_version)?;
        
        for header in &self.headers {
            write!(f, "{}\n", header)?;
        }

        write!(f, "\n")
    }
}