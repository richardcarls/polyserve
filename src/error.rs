use std::io;
use std::fmt;

pub struct Error(pub(crate) ErrorKind);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error(err.into())
  }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    ResolveBindAddr(io::Error),
    NoBindAddr,
    ResolveRootDir(io::Error),
    BindAddr(io::Error),
    IOError(io::Error),
    HttpParse,
    ResolveResource(&'static str),
    FeatureUnsupported(&'static str),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ResolveBindAddr(..) =>
                f.write_str("Could not resolve bind interface."),
            ErrorKind::NoBindAddr =>
                f.write_str("No suitable socket address found."),
            ErrorKind::ResolveRootDir(..) =>
                f.write_str("Could not resolve server root."),
            ErrorKind::BindAddr(..) =>
                f.write_str("Could not bind socket address."),
            ErrorKind::IOError(..) =>
                f.write_str("Encountered an unrecoverable I/O error."),
            ErrorKind::HttpParse =>
                f.write_str("Could not parse HTTP message."),
            ErrorKind::ResolveResource(ref msg) =>
                write!(f, "Could not resolve the server resource: {}", msg),
            ErrorKind::FeatureUnsupported(ref feature) =>
                write!(f, "Feature currently unsupported: {}.", feature),
        }
    }
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ErrorKind::ResolveBindAddr(ref err) => Some(err),
            ErrorKind::NoBindAddr => None,
            ErrorKind::ResolveRootDir(ref err) => Some(err),
            ErrorKind::BindAddr(ref err) => Some(err),
            ErrorKind::IOError(ref err) => Some(err),
            ErrorKind::HttpParse => None,
            ErrorKind::ResolveResource(_) => None,
            ErrorKind::FeatureUnsupported(_) => None,
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::IOError(err)
    }
}