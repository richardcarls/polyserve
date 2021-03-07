use std::io;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum ServerError {
    IOError(io::Error),
    NoBindAddr,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::IOError(..) => {
                write!(f, "IOError: {}", self.source().unwrap())
            },
            ServerError::NoBindAddr =>
                write!(f, "No suitable socket address found."),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ServerError::IOError(ref err) => Some(err),
            ServerError::NoBindAddr => None,
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> ServerError {
        ServerError::IOError(err)
    }
}