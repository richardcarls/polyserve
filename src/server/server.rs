use std::sync::Arc;
use std::default::Default;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

use super::ServerContext;

#[derive(Debug)]
pub struct Server<S: ServerState> {
    inner: S,
}

impl Server<Ready> {
    pub fn new() -> Server<Ready> {
        Server {
            inner: Ready::default(),
        }
    }

    pub fn set_ipv4(&mut self, ipv4: bool) {
        self.inner.ipv4 = ipv4;
    }

    pub fn with_interface(&mut self, interface: &str) {
        self.inner.interface = interface.to_owned();
    }

    pub fn with_port(&mut self, port: u16) {
        self.inner.port = port;
    }

    pub fn with_root(&mut self, root: &str) {
        self.inner.root = root.to_owned();
    }

    pub fn listen(self) -> Server<Listening> {
        let addr = (self.inner.interface.as_str(), self.inner.port).to_socket_addrs()
            .expect("Could not resolve bind interface IP")
            .find(|addr| {
                if self.inner.ipv4 == true && !addr.is_ipv4() {
                    false
                } else {
                    true
                }
            })
            .expect("No suitable bind interface found.");

        let root_dir = PathBuf::from(self.inner.root.as_str())
            .canonicalize()
            .expect("Could not resolve server root");

        println!("Serving {} at {}", root_dir.display(), addr);
        
        let context = Arc::new(ServerContext {
            addr,
            root_dir,
        });

        Server {
            inner: Listening {
                ready_state: self.inner,
                context,
            }
        }
    }
}

impl Server<Listening> {
    pub fn disconnect(self) -> Server<Ready> {
        Server {
            inner: self.inner.ready_state,
        }
    }
}

#[derive(Debug)]
pub struct Ready {
    ipv4: bool,
    interface: String,
    port: u16,
    root: String,
}

impl Default for Ready {
    fn default() -> Self {
        Self {
            ipv4: false,
            interface: String::from("::"),
            port: 8080,
            root: String::from("."),
        }
    }
}

#[derive(Debug)]
pub struct Listening {
    ready_state: Ready,
    context: Arc<ServerContext>,
}

pub trait ServerState {}
impl ServerState for Ready {}
impl ServerState for Listening {}