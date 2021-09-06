use std::default::Default;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::StreamExt;
use async_std::fs;
use async_std::task;
use async_std::net::{TcpListener, ToSocketAddrs};

use async_native_tls::TlsAcceptor;
use handlebars::Handlebars;

use crate::{Result, Error, ErrorKind, ServerContext};

const INDEX_TEMPLATE: &'static str = include_str!("../index.html.hbs");

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

    pub fn listen<'srv>(self) -> Result<()> {
        task::block_on(async {
            let addr = (self.inner.interface.as_str(), self.inner.port)
                .to_socket_addrs()
                .await
                .map_err(|err| Error(ErrorKind::ResolveBindAddr(err)))?
                .find(|addr| {
                    if self.inner.ipv4 == true && !addr.is_ipv4() {
                        false
                    } else {
                        true
                    }
                })
                .ok_or(Error(ErrorKind::NoBindAddr))?;

            let root_dir = PathBuf::from(self.inner.root.as_str())
                .canonicalize()?;

            println!("Serving {} at {}", root_dir.display(), addr);

            let mut hbs = Handlebars::new();

            // TODO: Support global index template override
            let _ = hbs.register_template_string("index", INDEX_TEMPLATE);

            // TLS
            // TODO: Put behind flag and configuration options for identity/key+cert
            let id_path = root_dir.join(Path::new(".identity.pfx"));
            let tls_acceptor = if id_path.is_file() {
                let id_file = fs::File::open(id_path).await?;

                // TODO: Identity file passwords besides empty string
                Some(TlsAcceptor::new(id_file, "").await?)
            } else {
                None
            };

            let tcp_listener = TcpListener::bind(addr).await?;

            let context = Arc::new(ServerContext {
                addr,
                root_dir,
                hbs,
                tcp_listener,
                tls_acceptor,
            });

            let server = Server {
                inner: Listening {
                    ready_state: self.inner,
                    context,
                },
            };

            server.inner.context.tcp_listener
                .incoming()
                .for_each_concurrent(None, |stream| async {
                    match stream {
                        Ok(stream) => {
                            let context = Arc::clone(&server.inner.context);

                            task::spawn(async move {
                                if let Some(tls_acceptor) = &context.tls_acceptor {
                                    if let Ok(tls_stream) = tls_acceptor.accept(&stream).await {
                                        if let Err(err) = context.handle_incoming(tls_stream).await {
                                            eprintln!("Uncaught error {:?}", err);
                                        };
                                    } else {
                                        if let Err(err) = context.handle_incoming(&stream).await {
                                            eprintln!("Uncaught error {:?}", err);
                                        };
                                    }
                                } else {
                                    if let Err(err) = context.handle_incoming(&stream).await {
                                        eprintln!("Uncaught error {:?}", err);
                                    };
                                };
                            });
                        },

                        Err(err) => eprintln!("Connection failed: {:?}", err),
                    }
                })
                .await;
            
            Ok(())
        })
    }
}

impl<'srv> Server<Listening<'srv>> {
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

pub struct Listening<'srv> {
    ready_state: Ready,
    context: Arc<ServerContext<'srv>>,
}

pub trait ServerState {}
impl ServerState for Ready {}
impl<'ctx> ServerState for Listening<'ctx> {}