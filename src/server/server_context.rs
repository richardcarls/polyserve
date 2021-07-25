use std::path::{Path, PathBuf};

use async_std::io;
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
            HttpMethod::Get => match self.resolve(request.path()) {
                Ok(resource) => {
                    println!("Secure request!");

                    resource.get_response().await?
                        .end(None, &mut stream).await;
                }
                Err(Error(ErrorKind::ResolveResource(_))) => {
                    Response::empty(404).end(None, &mut stream).await?;
                }
                Err(err) => {
                    return Err(err);
                }
            },

            _ => {
                Response::empty(405).end(None, &mut stream).await?;
            }
        };

        stream.flush().await.unwrap_or_default();

        Ok(())
    }

    pub(super) fn exists_file(&self, path: &Path) -> Option<PathBuf> {
        let rel_path: PathBuf = path.components().skip(1).collect();

        let abs_path = self.root_dir().join(rel_path);

        match abs_path.is_file() {
            true => Some(abs_path),
            false => None,
        }
    }

    pub(super) fn resolve(&self, path: &Path) -> Result<BoxedResource> {
        let supported_types = vec!["html", "md"];
        let rel_path: PathBuf = path.components().skip(1).collect();

        let abs_path = self.root_dir().join(rel_path);

        if abs_path.starts_with(self.root_dir()) == false {
            return Err(Error(ErrorKind::ResolveResource("Outside of server root!")));
        }

        // Priority 1: Explicit file match
        if let Ok(resource) = FileResource::from_path(abs_path.as_path()) {
            return Ok(resource.to_boxed());
        }

        // Priority 2: Implicit file match
        if abs_path.extension().is_none() {
            let implicit_path = supported_types.iter().find_map(|ext| {
                let test = format!("{}.{}", path.display(), ext);

                self.exists_file(Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if let Ok(resource) = FileResource::from_path(abs_path.as_path()) {
                    return Ok(resource.to_boxed());
                }
            }
        }

        // Priority 3: Implicit directory index
        // TODO: Resource set Content-Location header and 301 redirect to trailing slash URL
        if abs_path.is_dir() {
            let implicit_path = supported_types.iter().find_map(|ext| {
                let test = format!("{}/index.{}", path.display(), ext);

                self.exists_file(Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if let Ok(resource) = FileResource::from_path(abs_path.as_path()) {
                    return Ok(resource.to_boxed());
                }
            }
        }

        //Err(Error(ErrorKind::ResolveResource("Not found.")))
        Ok(EmptyResource::new().to_boxed())
    }
}
