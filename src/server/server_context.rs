use std::path::{ Path, PathBuf };
use std::str::FromStr;

use futures::io::BufReader;
use async_std::net::{ SocketAddr,  TcpStream };
use async_std::stream::StreamExt;
use async_std::io::prelude::*;
use async_std::fs;

use crate::*;
use crate::common::{ HttpMethod, HttpVersion, HttpHeader };

#[derive(Debug)]
pub(super) struct ServerContext {
    pub(super) addr: SocketAddr,
    pub(super) root_dir: PathBuf,
}

impl ServerContext {
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_path()
    }

    pub(super) async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let reader = BufReader::with_capacity(1024, &stream);
        let mut lines = reader.lines();

        let request_line = match lines.next().await {
            Some(line) => Ok(line),
            None => Err(Error(ErrorKind::HttpParseError)),
        }?
        .unwrap();

        let mut parts = request_line.split_whitespace();

        let method = parts.next();
        let path = parts.next();
        let http_version = parts.next();
        let headers = {
            let mut headers = Vec::new();

            while let Some(Ok(line)) = lines.next().await {
                if line.is_empty() {
                    // End of request header
                    break;
                }

                if let Ok(header) = HttpHeader::from_str(line.as_ref()) {
                    headers.push(header);
                }
            }

            headers
        };

        // TODO: request body

        let request = match (method, path, http_version) {
            (Some(method), Some(path), Some(http_version)) => {
                Ok(Request {
                    method: HttpMethod::from_str(method).unwrap(),
                    path: PathBuf::from(path),
                    http_version: HttpVersion::from_str(http_version).unwrap(),
                    headers,
                })
            },
            _ => Err(Error(ErrorKind::HttpParseError)),
        }?;

        println!("{}", request);

        let (response, body) = match (request.method, request.path) {
            (HttpMethod::Get, ref path) => {
                let rel_path = path.strip_prefix("/")
                    .unwrap_or(path);
                
                let abs_path = self.root_dir().join(rel_path);

                assert_eq!(abs_path.starts_with(self.root_dir()), true);

                match fs::read(abs_path).await {
                    Ok(data) => {
                        ("HTTP/1.1 200 Ok\r\nServer: polyserve (Rust)\r\n\r\n", Some(data))
                    },
                    Err(_) => ("HTTP/1.1 404 Not Found\r\nServer: polyserve (Rust)\r\n\r\n", None),
                }
            },
            _ => ("HTTP/1.1 405 Method Not Allowed\r\nServer: polyserve (Rust)\r\n\r\n", None),
        };
        
        stream.write(response.as_bytes()).await.unwrap_or_default();
        
        if let Some(body) = body {
            stream.write(&body).await.unwrap_or_default();
        }

        stream.flush().await.unwrap_or_default();

        Ok(())
    }
}