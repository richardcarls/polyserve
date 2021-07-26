use std::path::{Path, PathBuf};

use async_std::net::SocketAddr;
use futures::io::BufReader;
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::common::HttpMethod;
use crate::resource::*;
use crate::*;

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

    pub(super) async fn handle_connection<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin,
    {
        let mut reader = BufReader::with_capacity(1024, stream);
        let request = Request::from_reader(&mut reader).await?;

        println!("{}", request);

        let mut stream = reader.into_inner();

        match request.method() {
            HttpMethod::Get => match Resource::from_path(self.root_dir(), request.path()) {
                Ok(resource) => {
                    println!("Secure request!");

                    resource.respond(&mut stream).await?;
                }
                Err(Error(ErrorKind::ResolveResource(_))) => {
                    Response::new(404).send_empty(&mut stream).await?;
                }
                Err(err) => {
                    return Err(err);
                }
            },

            _ => {
                Response::new(405).send_empty(&mut stream).await?;
            }
        };

        stream.flush().await.unwrap_or_default();

        Ok(())
    }
}
