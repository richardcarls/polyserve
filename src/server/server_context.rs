use std::path::{Path, PathBuf};

use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use async_std::net::SocketAddr;

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

    pub(super) async fn handle_connection<S>(&self, mut stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin,
    {
        let request = Request::read_from(&mut stream).await?;

        println!("{}", request);

        match request.method() {
            HttpMethod::Get => match Resource::from_request(self.root_dir(), request) {
                Ok(resource) => resource.respond(&mut stream).await?,

                Err(Error(ErrorKind::ResolveResource(_))) => {
                    Response::new(404).send_empty(&mut stream).await?;
                },

                Err(err) => {
                    // Server and IO errors
                    return Err(err);
                },
            },

            _ => {
                Response::new(405).send_empty(&mut stream).await?;
            }
        };

        stream.flush().await?;

        Ok(())
    }
}
