mod common;
mod error;
mod server;
mod request;

pub use server::*;
pub use error::Error;

use error::ErrorKind;
use request::Request;

pub type Result<T> = std::result::Result<T, Error>;