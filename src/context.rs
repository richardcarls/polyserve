use std::sync::Arc;
use std::path::Path;

use async_std::net::SocketAddr;

use handlebars::Handlebars;

use crate::server::ServerContext;
use crate::{Request, Response};

#[derive(Debug)]
pub struct Context<'srv> {
    pub(crate) server_context: &'srv ServerContext<'srv>,
    pub request: Request,
    pub response: Response,
}

impl<'srv> Context<'srv> {
    pub fn addr(&self) -> &SocketAddr {
        &self.server_context.addr
    }

    pub fn root_dir(&self) -> &Path {
        self.server_context.root_dir.as_path()
    }

    pub fn hbs(&self) -> &Handlebars {
        &self.server_context.hbs
    }
}