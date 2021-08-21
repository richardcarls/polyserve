use std::path::{Path, PathBuf};

use futures::AsyncWrite;
use async_trait::async_trait;

use crate::{Error, ErrorKind, Request, Response, Result};
use super::{FileResource, IndexResource};

const SUPPORTED_TYPES: [&str; 2] = ["html", "md"];

pub enum Resource {
    None,
    File(FileResource),
    Index(IndexResource),
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

        let mut is_implicit = false;
        let mut is_index = false;

        // Priority 1: Explicit file match
        if abs_path.is_file() {
            let resource = FileResource {
                abs_path,
                url_path,
                is_implicit,
                is_index,
            };

            return Ok(Self::File(resource));
        }

        is_implicit = true;
        
        // Priority 2: Implicit file match
        if abs_path.extension().is_none() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}.{}", url_path, ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    let resource = FileResource {
                        abs_path,
                        url_path,
                        is_implicit,
                        is_index,
                    };
                    
                    return Ok(Self::File(resource));
                }
            }
        }

        is_index = true;

        // Priority 3: Implicit directory index
        if abs_path.is_dir() {
            let implicit_path = SUPPORTED_TYPES.iter().find_map(|ext| {
                let test = format!("{}/index.{}", url_path, ext);

                exists_file(root_dir, Path::new(test.as_str()))
            });

            if let Some(abs_path) = implicit_path {
                if abs_path.is_file() {
                    let resource = FileResource {
                        abs_path,
                        url_path,
                        is_implicit,
                        is_index,
                    };
                    
                    return Ok(Self::File(resource));
                }
            }

            // Priority 4: Generated directory index
            // TODO: configuration to allow/block generated indexes
            let resource = IndexResource {
                abs_path,
                url_path,
                is_implicit,
            };
            
            return Ok(Self::Index(resource));
        }

        Err(Error(ErrorKind::ResolveResource("Not found.")))
    }
}

#[async_trait]
impl Respond for Resource {
    async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send,
    {
        match self {
            Self::None => {
                Response::new(200).send_empty(stream).await
            },
            Self::File(resource) => resource.respond(stream).await,
            Self::Index(resource) => resource.respond(stream).await,
        }
    }
}

#[async_trait]
pub trait Respond {
    async fn respond<W>(self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send;
}


fn exists_file(root_dir: &Path, path: &Path) -> Option<PathBuf> {
    let rel_path: PathBuf = path.components().skip(1).collect();

    let abs_path = root_dir.join(rel_path);

    match abs_path.is_file() {
        true => Some(abs_path),
        false => None,
    }
}