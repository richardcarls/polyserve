use std::path::PathBuf;

use futures::AsyncWrite;
use async_trait::async_trait;

use crate::{ServerContext, Response, Result};
use super::{ResourceContext, Respond};

pub struct IndexResource {
    pub abs_path: PathBuf,
    pub url_path: String,
    pub is_implicit: bool,
    pub context: ResourceContext,
}

#[async_trait]
impl Respond for IndexResource {
    async fn respond<W>(self, context: &ServerContext, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        println!("Serving index for {}", self.url_path);

        let mut url_path = self.url_path;
        
        let dir_name = self.abs_path.components()
            .map(|seg| seg.as_os_str().to_str().unwrap_or(""))
            .last()
            .unwrap_or("/");

        let html = context.hbs.render("index", &self.context)?;
        
        let mut response = match url_path.ends_with("/") {
            true => Response::new(200),
            false => {
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
            }
        };

        response.set_header("Content-Type", vec!["text/html; charset=UTF-8".to_owned()]);
        
        response.send_str(html.as_str(), stream).await
    }
}