mod common;
mod error;
mod server;
mod context;
mod request;
mod response;
mod resource;

pub use server::*;
pub use error::Error;

use error::ErrorKind;
use context::Context;
use request::Request;
use response::Response;

pub type Result<T> = std::result::Result<T, Error>;