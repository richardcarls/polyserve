use std::path::PathBuf;
use std::sync::Arc;
use std::default::Default;

use futures::stream::StreamExt;
use async_std::task;
use async_std::net::{ TcpListener, ToSocketAddrs };

use super::ServerContext;
use crate::{ Result, Error, ErrorKind, Response };

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

    pub fn listen(self) -> Result<Server<Listening>> {
        task::block_on(async {
            let addr = (self.inner.interface.as_str(), self.inner.port)
                .to_socket_addrs().await
                .map_err(|err| Error(ErrorKind::ResolveBindAddr(err)))?
                .find(|addr| {
                    if self.inner.ipv4 == true && !addr.is_ipv4() {
                        false
                    } else {
                        true
                    }
                }).ok_or(Error(ErrorKind::NoBindAddr))?;

            let root_dir = PathBuf::from(self.inner.root.as_str())
                .canonicalize()
                .map_err(|err| Error(ErrorKind::ResolveRootDir(err)))?;

            println!("Serving {} at {}", root_dir.display(), addr);
            
            let context = Arc::new(ServerContext {
                addr,
                root_dir,
            });

            let tcp_listener = TcpListener::bind(addr).await
                .map_err(|err| Error(ErrorKind::BindAddr(err)))?;
            
            let server = Server {
                inner: Listening {
                    ready_state: self.inner,
                    context,

                    tcp_listener,
                },
            };

            server.inner.tcp_listener.incoming()
                .for_each_concurrent(None, |stream| async {
                    if let Ok(mut stream) = stream {
                        let context = Arc::clone(&server.inner.context);
                    
                        task::spawn(async move {
                            if let Err(err) = context.handle_connection(&mut stream).await {
                                eprintln!("Unhandled Error: {:?}", err);

                                let _ = Response::new(500).send_empty(&mut stream).await;
                            };
                        });
                    }
                }).await;

            Ok(server)
        })
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

    tcp_listener: TcpListener,
}

pub trait ServerState {}
impl ServerState for Ready {}
impl ServerState for Listening {}