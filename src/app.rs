use std::io;
use std::fs;
use std::net::ToSocketAddrs;
use std::error::Error as StdError;
use std::path::{PathBuf, Path};

use roa;
use roa::preload::*;
use roa::tls::{TlsListener, ServerConfig, NoClientAuth};
use roa::tls::internal::pemfile::{certs, pkcs8_private_keys};

use crate::PolyState;
use crate::middleware;

// TODO: Remove roa dependency (use hyper directly, refactor middleware fns)
pub struct App {}

impl App {
    pub async fn listen(&self, interface: impl ToSocketAddrs, root_path: &Path) -> Result<(), Box<dyn StdError>> {
        let addr = interface
            .to_socket_addrs()?
            .find(|addr| addr.is_ipv4())
            .unwrap();
        
        let root = PathBuf::from(root_path).canonicalize()?;

        // TODO: Config for Cert path, default "./identity/server.crt"
        // TODO: Config for Key path, default "./identity/server.key"
        let id_path = root.join(Path::new(".identity"));
        let cert_path = id_path.join("server.crt");
        let key_path = id_path.join("server.key");

        let config = match (cert_path.is_file(), key_path.is_file()) {
            (true, true) => {
                let mut config = ServerConfig::new(NoClientAuth::new());

                let mut cert_file = io::BufReader::new(fs::File::open(cert_path)?);
                let mut key_file = io::BufReader::new(fs::File::open(key_path)?);

                let cert_chain = certs(&mut cert_file);

                // TODO: Config flag for RSA or PKCS8 key file, default PKCS8
                let keys = pkcs8_private_keys(&mut key_file);
                match (cert_chain, keys) {
                    (Ok(cert_chain), Ok(mut keys)) if keys.len() > 0 => {
                        config.set_single_cert(cert_chain, keys.remove(0))?;

                        Some(config)
                    },

                    _ => None,
                }
            },
            _ => None,
        };

        let state = PolyState {
            addr,
            root,
        };

        let app = roa::App::state(state)
            // TODO: Custom logger middleware
            .gate(middleware::logger)
            .gate(middleware::early_return)
            .gate(middleware::server_header)
            .gate(middleware::resolve_resource)
            .gate(middleware::allow_methods)
            .gate(middleware::trailing_slash)
            .gate(middleware::serve_file)
            .gate(middleware::render_hbs)
            .gate(middleware::use_index)
            .gate(middleware::resolve_file)
            .gate(middleware::auto_index)
            .end(());

        // TODO: Graceful shutdown
        if let Some(config) = config {
            app.listen_tls(addr, config, |addr| {
                log::info!("Serving {:?} over https on {}", root_path, addr)
            })?
            .await?;
        } else {
            app.listen(addr, |addr| {
                log::info!("Serving {:?} over http on {}", root_path, addr)
            })?
            .await?;
        }

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}
