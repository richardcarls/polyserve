use std::path::PathBuf;

use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use async_std::net::{TcpListener, TcpStream, SocketAddr};
use async_native_tls::TlsAcceptor;
use handlebars::Handlebars;

use crate::common::*;
use crate::resource::{Respond, Resource};
use crate::{Result, Error, ErrorKind, Context, Request, Response};

#[derive(Debug)]
pub(crate) struct ServerContext<'srv> {
    pub(crate) addr: SocketAddr,
    pub(crate) root_dir: PathBuf,
    pub(crate) hbs: Handlebars<'srv>,
    pub(crate) tcp_listener: TcpListener,
    pub(crate) tls_acceptor: Option<TlsAcceptor>,
}

impl<'srv> ServerContext<'srv> {
    pub(crate) async fn handle_incoming<S>(&self, mut stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin,
    {
        let request = Request::read_from(&mut stream).await?;
        let context = Context {
            server_context: &self,
            request,
            response: Response::default(),
        };
        
        self.serve(stream, context).await
    }

    pub(crate) async fn serve<S>(&self, mut stream: S, ctx: Context<'srv>) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin,
    {
        //let Context { request, response, .. } = ctx;

        println!("{}", ctx.request);

        match ctx.request.method() {
            HttpMethod::Get => match Resource::resolve(&ctx) {
                Ok(resource) => resource.respond(&ctx, &mut stream).await?,

                Err(Error(ErrorKind::ResolveResource(_))) => {
                    Response::new(404).send_empty(&mut stream).await?;
                },

                Err(err) => {
                    // Server and IO errors
                    return Err(err);
                },
            },

            _ => {
                Response::new(405).send_empty(&mut stream).await?;
            }
        };

        stream.flush().await?;

        Ok(())
    }
}