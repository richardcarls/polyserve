use std::path::{ Path, PathBuf };
use async_std::net::{ SocketAddr,  TcpStream };
use async_std::io::prelude::*;
use async_std::fs;

use crate::*;
use crate::common::HttpMethod;

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

    pub(super) async fn handle_connection(&self, stream: &mut TcpStream) -> Result<()> {
        let request = Request::from_stream(&stream).await?;

        println!("{}", request);

        let resource_path = self.resolve(request.path());

        match (resource_path, request.method()) {
            (Ok(resource_path), HttpMethod::Get) => {
                match fs::File::open(resource_path).await {
                    Ok(mut file) => {
                        Response::new(200).send_file(&mut file, stream).await?;
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        Response::new(404).send_empty(stream).await?;
                    },
                }
            },

            (Err(err), _) => {
                eprintln!("{}", err);

                Response::new(404).send_empty(stream).await?;
            },
            
            _ => {
                Response::new(405).send_empty(stream).await?;
            },
        };

        stream.flush().await.unwrap_or_default();

        Ok(())
    }

    pub(super) fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let supported_types = vec!("html", "md");
        let abs_path = url_to_abs_path(self.root_dir(), path);

        if abs_path.starts_with(self.root_dir()) == false {
            return Err(Error(ErrorKind::ResolveResource("Outside of server root!")))
        }

        // Priority 1: Explicit file match
        if abs_path.is_file() {
            return Ok(abs_path)
        }

        let implicit_path = supported_types.iter()
            .find_map(|ext| {
                let test = format!("{}.{}", path.display(), ext);

                self.resolve(Path::new(test.as_str())).ok()
            });

        
        if let Some(abs_path) = implicit_path {
            return Ok(abs_path)
        }

        // Priority 2: Implicit file match
        if abs_path.extension().is_none() {
            let implicit_path = supported_types.iter()
                .find_map(|ext| {
                    let test = format!("{}.{}", path.display(), ext);

                    self.resolve(Path::new(test.as_str())).ok()
                });

            
            if let Some(abs_path) = implicit_path {
                return Ok(abs_path)
            }
        }

        // Priority 3: Implicit directory index
        if abs_path.is_dir() {
            let implicit_path = supported_types.iter()
                .find_map(|ext| {
                    let test = format!("{}/index.{}", path.display(), ext);

                    self.resolve(Path::new(test.as_str())).ok()
                });

            
            if let Some(abs_path) = implicit_path {
                return Ok(abs_path)
            }
        }

        Err(Error(ErrorKind::ResolveResource("Not found.")))
    }
}

fn url_to_abs_path(root: &Path, url: &Path) -> PathBuf {
    let rel_path: PathBuf = url.components()
        .skip(1)
        .collect();

    let abs_path = root.join(rel_path);

    abs_path
}