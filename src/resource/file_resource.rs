use std::path::{Path, PathBuf};

use futures::AsyncWrite;
use async_std::fs;
use async_trait::async_trait;
use mime;
use mime_guess;

use crate::{Context, Response, Result};
use super::{ResourceContext, Respond};

pub struct FileResource {
    pub abs_path: PathBuf,
    pub url_path: String,
    pub is_implicit: bool,
    pub is_index: bool,
    pub context: ResourceContext,
}

#[async_trait]
impl Respond for FileResource {
    async fn respond<W>(self, ctx: &Context, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let abs_path = self.abs_path.as_path();
        let mut url_path = self.url_path;

        let mime_type = mime_guess::from_path(abs_path)
            .first()
            .unwrap_or(mime::TEXT_PLAIN_UTF_8);
        
        let mime_type = mime_type.essence_str();
        
        let file = fs::File::open(abs_path).await?;

        let mut response = match self.is_index {
            true => {
                if url_path.ends_with("/") == false {
                    // Add explicit trailing slash to location header
                    url_path.push('/');

                    let mut response = Response::new(301);

                    // Tell client to redirect
                    response.set_header("Location", &[url_path.as_str()]);

                    // Tell client actual location of file
                    // TODO: For implicit index pages, give full file poth here
                    response.set_header("Content-Location", &[url_path.as_str()]);

                    response
                } else {
                    Response::new(200)
                }
            },

            false => Response::new(200)
        };

        response.set_header("Content-Type", &[mime_type]);

        response.send_file(file, stream).await
    } 
}