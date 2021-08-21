use std::path::{Path, PathBuf};

use futures::AsyncWrite;
use async_std::fs;
use async_trait::async_trait;
use mime;
use mime_guess;

use crate::{Response, Result};
use super::Respond;

pub struct FileResource {
    pub abs_path: PathBuf,
    pub url_path: String,
    pub is_implicit: bool,
    pub is_index: bool,
}

#[async_trait]
impl Respond for FileResource {
    async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let abs_path = self.abs_path.as_path();
        let mut url_path = self.url_path;

        let mime_type = mime_guess::from_path(abs_path)
            .first()
            .unwrap_or(mime::TEXT_PLAIN_UTF_8)
            .essence_str()
            .to_owned();
        
        let file = fs::File::open(abs_path).await?;

        let mut response = match self.is_index {
            true => {
                if url_path.ends_with("/") == false {
                    // Add explicit trailing slash to location header
                    url_path.push('/');

                    let mut response = Response::new(301);

                    // Tell client to redirect
                    response.set_header(
                        "Location",
                        vec![url_path.to_owned()]
                    );

                    // Tell client actual location of file
                    // TODO: For implicit index pages, give full file poth here
                    response.set_header(
                        "Content-Location",
                        vec![url_path.to_owned()]
                    );

                    response
                } else {
                    Response::new(200)
                }
            },

            false => Response::new(200)
        };

        response.set_header("Content-Type", vec![mime_type.to_owned()]);

        response.send_file(file, stream).await
    } 
}