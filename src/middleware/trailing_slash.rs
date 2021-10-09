use roa::{Context, Next, Result, status, http};

use crate::{Resource, ServerConfig};

pub async fn trailing_slash<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        let ServerConfig { ref force_trailing_slash, .. } = res.config().server;

        if res.fs_path().is_dir() && !ctx.uri().path().ends_with("/") && *force_trailing_slash {
            // Add explicit trailing slash to location header
            ctx.resp.headers.insert("Location", format!("{}/", ctx.uri().path()).parse()?);
                                    
            return Err(status!(http::StatusCode::MOVED_PERMANENTLY));
        }
    }

    next.await
}