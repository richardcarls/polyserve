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
        let abs_path = url_to_abs_path(self.root_dir(), path);

        if abs_path.starts_with(self.root_dir()) == true {
            match (abs_path.is_file(), abs_path.is_dir()) {
                (true, _) => Ok(abs_path),
                (_, true) => {
                    //Err(Error(ErrorKind::FeatureUnsupported("DirectoryIndex")))
                    let index_path = format!("{}/index.html", path.display());

                    self.resolve(Path::new(index_path.as_str()))
                }
                _ => {
                    if abs_path.extension().is_none() {
                        let supported_types = vec!("html", "md");

                        let file_path = supported_types.iter()
                            .map(|ext| {
                                let test = format!("{}.{}", path.display(), ext);
                                url_to_abs_path(self.root_dir(), Path::new(test.as_str()))
                            })
                            .find(|file_path| {
                                file_path.is_file() && file_path.starts_with(self.root_dir())
                            });
                        
                        if let Some(file_path) = file_path {
                            Ok(file_path)
                        } else {
                            Err(Error(ErrorKind::ResolveResource("Not found.")))
                        }
                    } else {
                        Err(Error(ErrorKind::ResolveResource("Not found.")))
                    }
                },
            }
        } else {
            Err(Error(ErrorKind::ResolveResource("Outside of server root!")))
        }
    }
}

fn url_to_abs_path(root: &Path, url: &Path) -> PathBuf {
    let rel_path: PathBuf = url.components()
        .skip(1)
        .collect();

    let abs_path = root.join(rel_path);

    abs_path
}