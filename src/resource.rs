use std::path::{Path, PathBuf};

use async_std::fs;
use async_std::io;
use futures::{AsyncRead, AsyncWrite};

use async_trait::async_trait;

use mime;
use mime_guess;

use crate::{Error, ErrorKind, Response, Result};

pub type BoxedResource = Box<dyn Resource>;

pub struct EmptyResource {}

impl EmptyResource {
    pub fn new() -> Self {
        EmptyResource {}
    }

    pub fn to_boxed(self) -> BoxedResource {
        Box::new(self)
    }
}

#[async_trait]
impl Resource for EmptyResource {
    async fn get_response(&self) -> Result<Response> {
        Ok(Response::empty(404))
    }
}

pub struct FileResource {
    pub(crate) abs_path: PathBuf,
    pub(crate) mime_type: String,
}

impl FileResource {
    pub fn from_path(abs_path: &Path) -> Result<FileResource> {
        if abs_path.is_file() {
            let mime_type = mime_guess::from_path(abs_path)
                .first()
                .unwrap_or(mime::TEXT_PLAIN_UTF_8)
                .essence_str()
                .to_owned();

            Ok(FileResource {
                abs_path: abs_path.to_owned(),
                mime_type,
            })
        } else {
            Err(Error(ErrorKind::ResolveResource("File not found.")))
        }
    }

    pub fn to_boxed(self) -> BoxedResource {
        Box::new(self)
    }
}

#[async_trait]
impl Resource for FileResource {
    async fn get_response(&self) -> Result<Response> {
        let mut file = fs::File::open(self.abs_path.as_path()).await?;

        let mut response = Response::from_file(200, file).await?;
        response.set_header("Content-Type", vec![self.mime_type.to_owned()]);

        Ok(response)
    }
}

#[async_trait]
pub trait Resource: Send + Sync {
    async fn get_response(&self) -> Result<Response>;
    /*
    async fn respond<W>(&self, stream: &mut W) -> Result<()>
    where
        Self: Sized,
        W: AsyncWrite + Send + Unpin,;
    */
}