use std::path::{Path, PathBuf};

use async_std::fs;
use futures::AsyncWrite;

use mime;
use mime_guess;

use crate::{Error, ErrorKind, Request, Response, Result};

const SUPPORTED_TYPES: [&str; 2] = ["html", "md"];

pub enum Resource {
    Empty,
    File { abs_path: PathBuf, url_path: String, is_index: bool },
}

impl Resource {
    pub fn from_request(root_dir: &Path, request: Request) -> Result<Self> {
        let url_path = request.path().to_owned();

        let rel_path: PathBuf = PathBuf::from(request.path())
            .components() // Note: Normalizes away trailing slash
            .skip(1) // Remove leading slash
            .collect();

        let abs_path = root_dir.join(rel_path);

        if abs_path.starts_with(root_dir) == false {
            return Err(Error(ErrorKind::ResolveResource("Outside of root dir!")));
        }

        // Priority 1: Explicit file match
        if abs_path.is_file() {
            return Ok(Self::File { abs_path, url_path, is_index: false });
        }

        // Priority 2: Implicit file match
        if abs_path.extension().is_none() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}.{}", url_path, ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    return Ok(Self::File{ abs_path, url_path, is_index: false });
                }
            }
        }

        // Priority 3: Implicit directory index
        if abs_path.is_dir() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}/index.{}", url_path, ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    return Ok(Self::File{ abs_path, url_path, is_index: true });
                }
            }
        }

        // TODO: Priority 4: Generated Index

        Err(Error(ErrorKind::ResolveResource("Not found.")))
    }

    pub async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self {
            Self::Empty => {
                Response::new(200).send_empty(stream).await
            },

            Self::File{ abs_path, mut url_path, is_index }  => {
                let abs_path = abs_path.as_path();

                let mime_type = mime_guess::from_path(abs_path)
                    .first()
                    .unwrap_or(mime::TEXT_PLAIN_UTF_8)
                    .essence_str()
                    .to_owned();
                
                let file = fs::File::open(abs_path).await?;

                let mut response = match is_index {
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
            },
        }
    }
}



fn exists_file(root_dir: &Path, path: &Path) -> Option<PathBuf> {
    let rel_path: PathBuf = path.components().skip(1).collect();

    let abs_path = root_dir.join(rel_path);

    match abs_path.is_file() {
        true => Some(abs_path),
        false => None,
    }
}