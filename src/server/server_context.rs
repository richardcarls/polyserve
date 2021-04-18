use std::path::{ Path, PathBuf };
use async_std::net::{ SocketAddr,  TcpStream };
use async_std::io::prelude::*;
use async_std::fs;

use crate::*;
use crate::common::{ HttpMethod, HttpStatusCode };

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

    pub(super) async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let request = Request::from_stream(&stream).await?;

        println!("{}", request);

        match request.method() {
            HttpMethod::Get => {
                let rel_path: PathBuf = request.path().components()
                    .skip(1)
                    .collect();

                let abs_path = self.root_dir().join(rel_path);

                assert_eq!(abs_path.starts_with(self.root_dir()), true);

                match fs::File::open(abs_path).await {
                    Ok(mut file) => {
                        Response::new(200).send_file(&mut file, &mut stream).await?;
                    },
                    Err(_) => {
                        Response::new(404).send(&mut stream).await?;
                    },
                }
            },
            
            _ => {
                Response::new(405).send(&mut stream).await?;
            },
        };

        stream.flush().await.unwrap_or_default();

        Ok(())
    }
}