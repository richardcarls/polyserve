use std::path::{Path, PathBuf};

use async_std::io::prelude::*;
use async_std::net::{SocketAddr, TcpStream};

use crate::common::HttpMethod;
use crate::*;
use crate::resource::*;

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

    match request.method() {
      HttpMethod::Get => {
        match self.resolve(request.path()) {
          Ok(resource) => {
            resource.respond(stream).await?;
          },
          Err(Error(ErrorKind::ResolveResource(_))) => {
            Response::new(404).send_empty(stream).await?;
          },
          Err(err) => {
            return Err(err);
          },
        }
      },
      
      _ => {
        Response::new(405).send_empty(stream).await?;
      },
    };

    stream.flush().await.unwrap_or_default();

    Ok(())
  }

  pub(super) fn resolve (&self, path: &Path) -> Result<BoxedResource> {
    let supported_types = vec!["html", "md"];
    let rel_path: PathBuf = path.components().skip(1).collect();

    let abs_path = self.root_dir().join(rel_path);

    if abs_path.starts_with(self.root_dir()) == false {
      return Err(Error(ErrorKind::ResolveResource("Outside of server root!")));
    }

    // Priority 1: Explicit file match
    if let Ok(resource) = FileResource::from_path(abs_path.as_path()) {
      return Ok(resource.to_boxed());
    }

    // Priority 2: Implicit file match
    if abs_path.extension().is_none() {
      let implicit_resource = supported_types.iter().find_map(|ext| {
        let test = format!("{}.{}", path.display(), ext);

        self.resolve(Path::new(test.as_str())).ok()
      });

      if let Some(boxed_resource) = implicit_resource {
        return Ok(boxed_resource);
      }
    }

    // Priority 3: Implicit directory index
    // TODO: Resource set Content-Location header and 301 redirect to trailing slash URL
    if abs_path.is_dir() {
      let implicit_resource = supported_types.iter().find_map(|ext| {
        let test = format!("{}/index.{}", path.display(), ext);

        self.resolve(Path::new(test.as_str())).ok()
      });

      if let Some(boxed_resource) = implicit_resource {
        return Ok(boxed_resource);
      }
    }

    Err(Error(ErrorKind::ResolveResource("Not found.")))
  }
}
