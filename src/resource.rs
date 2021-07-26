use std::path::{Path, PathBuf};

use async_std::fs;
use futures::AsyncWrite;

use mime;
use mime_guess;

use crate::{Error, ErrorKind, Response, Result};

const SUPPORTED_TYPES: [&str; 2] = ["html", "md"];

pub enum Resource {
    Empty,
    File(PathBuf),
}

impl Resource {
    pub fn from_path(root_dir: &Path, path: &Path) -> Result<Self> {
        let rel_path: PathBuf = path.components().skip(1).collect();

        let abs_path = root_dir.join(rel_path);

        if abs_path.starts_with(root_dir) == false {
            return Err(Error(ErrorKind::ResolveResource("Outside of root dir!")));
        }

        // Priority 1: Explicit file match
        if abs_path.is_file() {
            return Ok(Self::File(abs_path));
        }

        // Priority 2: Implicit file match
        if abs_path.extension().is_none() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}.{}", path.display(), ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    return Ok(Self::File(abs_path));
                }
            }
        }

        // Priority 3: Implicit directory index
        // TODO: Resource set Content-Location header and 301 redirect to trailing slash URL
        if abs_path.is_dir() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}/index.{}", path.display(), ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    return Ok(Self::File(abs_path));
                }
            }
        }

        //Err(Error(ErrorKind::ResolveResource("Not found.")))
        Ok(Self::Empty)
    }

    pub async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self {
            Self::Empty => {
                Response::new(200).send_empty(stream).await
            },

            Self::File(ref abs_path) => {
                let mime_type = mime_guess::from_path(abs_path)
                    .first()
                    .unwrap_or(mime::TEXT_PLAIN_UTF_8)
                    .essence_str()
                    .to_owned();
                
                let file = fs::File::open(abs_path.as_path()).await?;

                let mut response = Response::new(200);

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