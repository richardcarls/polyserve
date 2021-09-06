use std::fmt;
use std::io;

use async_native_tls;
use handlebars;
use serde_json;

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

impl From<async_native_tls::Error> for Error {
    fn from(err: async_native_tls::Error) -> Error {
        Error(err.into())
    }
}

impl From<async_native_tls::AcceptError> for Error {
    fn from(err: async_native_tls::AcceptError) -> Error {
        Error(err.into())
    }
}

impl From<handlebars::RenderError> for Error {
    fn from(err: handlebars::RenderError) -> Error {
        Error(err.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error(err.into())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error(err.into())
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    NoBindAddr,
    ResolveBindAddr(io::Error),
    TlsError(async_native_tls::Error),
    TlsAcceptError(async_native_tls::AcceptError),
    IOError(io::Error),
    HttpParse,
    ResolveResource(&'static str),
    JsonParse(serde_json::Error),
    RenderError(handlebars::RenderError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::NoBindAddr => f.write_str("No suitable socket address found."),
            ErrorKind::ResolveBindAddr(..) => f.write_str("Could not resolve bind interface."),
            ErrorKind::TlsError(ref err) => write!(f, "TLS Error: {:?}", err),
            ErrorKind::TlsAcceptError(ref err) => write!(f, "TLS AcceptError: {:?}", err),
            ErrorKind::IOError(..) => f.write_str("Encountered an unrecoverable I/O error."),
            ErrorKind::HttpParse => f.write_str("Could not parse HTTP message."),
            ErrorKind::ResolveResource(ref msg) => {
                write!(f, "Could not resolve the server resource: {}", msg)
            }
            ErrorKind::JsonParse(ref err) => write!(f, "Json parse error: {:?}", err),
            ErrorKind::RenderError(ref err) => write!(f, "Handlebars render error: {:?}", err),
        }
    }
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ErrorKind::NoBindAddr => None,
            ErrorKind::ResolveBindAddr(ref err) => Some(err),
            ErrorKind::TlsError(ref err) => Some(err),
            ErrorKind::TlsAcceptError(ref err) => Some(err),
            ErrorKind::IOError(ref err) => Some(err),
            ErrorKind::HttpParse => None,
            ErrorKind::ResolveResource(_) => None,
            ErrorKind::JsonParse(ref err) => Some(err),
            ErrorKind::RenderError(ref err) => Some(err),
        }
    }
}

impl From<async_native_tls::Error> for ErrorKind {
    fn from(err: async_native_tls::Error) -> ErrorKind {
        ErrorKind::TlsError(err)
    }
}

impl From<async_native_tls::AcceptError> for ErrorKind {
    fn from(err: async_native_tls::AcceptError) -> ErrorKind {
        ErrorKind::TlsAcceptError(err)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> ErrorKind {
        ErrorKind::JsonParse(err)
    }
}

impl From<handlebars::RenderError> for ErrorKind {
    fn from(err: handlebars::RenderError) -> ErrorKind {
        ErrorKind::RenderError(err)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::IOError(err)
    }
}
