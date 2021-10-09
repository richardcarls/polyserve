use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PolyState {
    pub(crate) addr: SocketAddr,
    pub(crate) root: PathBuf,
}

impl PolyState {
    #[allow(dead_code)]
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }
    
    pub fn root_path(&self) -> &Path {
        self.root.as_path()
    }
}