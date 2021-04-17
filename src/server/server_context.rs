use std::path::{ Path, PathBuf };
use std::str::FromStr;

use futures::io::BufReader;
use async_std::net::{ SocketAddr,  TcpStream };
use async_std::stream::StreamExt;
use async_std::io::prelude::*;
use async_std::fs;

use crate::*;
use crate::common::{ HttpMethod, HttpHeader };

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

        let mut request = Request::from_request_line(request_line.as_ref())?;

        while let Some(Ok(line)) = lines.next().await {
            if line.is_empty() {
                // End of request header
                break;
            }

            if let Ok(header) = HttpHeader::from_str(line.as_ref()) {
                request.headers.push(header);
            }
        }

        // TODO: request body

        println!("{}", request);

        let (response, body) = match (request.method(), request.path()) {
            (HttpMethod::Get, ref path) => {
                let rel_path: PathBuf = path.components().skip(1).collect();

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