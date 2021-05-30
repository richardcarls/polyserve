use std::path::{Path, PathBuf};

use async_std::fs;
use async_std::net::TcpStream;

use mime;
use mime_guess;

use crate::{Result, Error, ErrorKind, Response};

pub struct FileResource {
  pub(crate) abs_path: PathBuf,
  pub(crate) mime_type: String,
}

impl FileResource {
  pub fn from_path(abs_path: &Path) -> Result<FileResource> {
    if abs_path.is_file() {
      let mime_type = mime_guess::from_path(abs_path).first()
        .unwrap_or(mime::TEXT_PLAIN_UTF_8)
        .essence_str()
        .to_owned();

      Ok(FileResource {
        abs_path: abs_path.to_owned(),
        mime_type,
      })
    } else {
      Err(Error(ErrorKind::ResolveResource("File not found.")))
    }
  }

  pub async fn respond(self, stream: &mut TcpStream) -> Result<()> {
    let mut file = fs::File::open(self.abs_path.as_path()).await
      .map_err(|err| Error(ErrorKind::IOError(err)))?;

    let mut response = Response::new(200);
    response.set_header("Content-Type", vec![self.mime_type]);

    response.send_file(&mut file, stream).await
  }
}