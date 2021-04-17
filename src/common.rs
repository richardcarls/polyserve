use std::str::FromStr;
use std::fmt;

// TODO: Wrap in struct to support new methods without major version bump
// @See ServerError

// TODO: Support more methods
#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Other(String)
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<HttpMethod, ()> {
        Ok(match s {
            s if s.eq_ignore_ascii_case("GET") => HttpMethod::Get,
            s => HttpMethod::Other(s.to_ascii_uppercase().to_owned()),
        })
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            HttpMethod::Other(method) => method.to_ascii_uppercase(),
            _ => format!("{:?}", self).to_ascii_uppercase(),
        })
    }
}

#[derive(Debug)]
pub struct HttpVersion(pub u8, pub u8);

impl FromStr for HttpVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<HttpVersion, ()> {
        match s.strip_prefix("HTTP/") {
            Some(version_string) => {
                let parts = version_string
                    .splitn(2, '.')
                    .map(|s| {
                        s.parse().map_err(|_| ())
                    })
                    .collect::<Vec<Result<u8, ()>>>();

                let parts = (parts[0], parts[1]);

                Ok(match parts {
                    (Ok(major), Ok(minor)) => HttpVersion(major, minor),
                    _ => HttpVersion(1, 1),
                })
            },
            None => Ok(HttpVersion(1, 1)),
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP/{}.{}", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct HttpHeader {
    pub name: String,
    pub values: Vec<String>,
}

impl FromStr for HttpHeader {
    type Err = ();

    fn from_str(s: &str) -> Result<HttpHeader, ()> {
        let mut parts = s.splitn(2, ':');

        let name = parts.next();
        let values = parts.next();

        match (name, values) {
            (Some(name), Some(values)) => {
                let name = name.trim().to_owned();

                let values: Vec<String> = values
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .collect();

                Ok(HttpHeader {
                    name, 
                    values,
                })
            },
            _ => Err(()),
        }
    }
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.values.join(","))
    }
}