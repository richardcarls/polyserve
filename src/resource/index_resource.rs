use std::path::{Path, PathBuf};

use futures::AsyncWrite;
use async_trait::async_trait;
use serde_json;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;

use crate::{Error, ErrorKind, Response, Result};
use super::Respond;

const INDEX_TEMPLATE: &'static str = include_str!("../index.html.hbs");

pub struct IndexResource {
    pub abs_path: PathBuf,
    pub url_path: String,
    pub is_implicit: bool,
}

#[async_trait]
impl Respond for IndexResource {
    async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        println!("Serving index for {}", self.url_path);

        // TODO: ServerContext hbs register
        let mut hbs = Handlebars::new();

        let mut url_path = self.url_path;
        
        let dir_name = self.abs_path.components()
            .map(|seg| seg.as_os_str().to_str().unwrap_or(""))
            .last()
            .unwrap_or("/");
        
        let entries: Vec<serde_json::Value> = self.abs_path.read_dir()?
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let unknown = "(unknown)".to_owned();

                let entry_name = entry.file_name().to_owned().into_string()
                    .unwrap_or(unknown.clone());

                serde_json::json!({
                    "name": entry_name,
                    "abs_path": entry.path(),
                    "metadata": match entry.metadata() {
                        Ok(meta) => {
                            let modified = match meta.modified() {
                                Ok(st) => {
                                    let dt: DateTime<Utc> = st.clone().into();

                                    format!("{}", dt.format("%+"))
                                },
                                _ => unknown.clone()
                            };

                            let accessed = match meta.accessed() {
                                Ok(st) => {
                                    let dt: DateTime<Utc> = st.clone().into();

                                    format!("{}", dt.format("%+"))
                                },
                                _ => unknown.clone()
                            };

                            let created = match meta.created() {
                                Ok(st) => {
                                    let dt: DateTime<Utc> = st.clone().into();

                                    format!("{}", dt.format("%+"))
                                },
                                _ => unknown.clone()
                            };

                            serde_json::json!({
                                "is_dir": meta.is_dir(),
                                "is_file": meta.is_file(),
                                "len": meta.len(),
                                "readonly": meta.permissions().readonly(),
                                "modified": modified,
                                "accessed": accessed,
                                "created": created,
                            })
                        },

                        Err(_) => serde_json::json!({}),
                    },
                })
            })
            .collect();
        
        // TODO: Current dir metadata
        // TODO: Ancestors
        let context = serde_json::json!({
            "name": dir_name,
            "entries": entries,
        });

        println!("{}", context);

        let html = hbs.render_template(INDEX_TEMPLATE, &context)
            .map_err(|_| Error(ErrorKind::ResolveResource("")))?;
        
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