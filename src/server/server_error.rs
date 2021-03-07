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

#[derive(Debug)]
pub(crate) enum ErrorKind {
    ResolveBindAddr(io::Error),
    NoBindAddr,
    ResolveRootDir(io::Error),
    IOError(io::Error),
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
            ErrorKind::IOError(..) =>
                f.write_str("Encountered an unrecoverable I/O error."),
        }
    }
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ErrorKind::ResolveBindAddr(ref err) => Some(err),
            ErrorKind::NoBindAddr => None,
            ErrorKind::ResolveRootDir(ref err) => Some(err),
            ErrorKind::IOError(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::IOError(err)
    }
}