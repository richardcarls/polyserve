use std::sync::Arc;

use mime;
use roa::{Context, Next, Result, status, http};

use handlebars::Handlebars;

use crate::{Resource, ServerConfig};

pub async fn auto_index<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        let ServerConfig { ref auto_index, .. } = res.config().server;

        if res.fs_path().is_dir() && *auto_index {
            let hbs = ctx.load::<Arc<Handlebars>>("hbs");

            if let Some(hbs) = hbs.as_deref() {
                let html = hbs.render("index", res.context())?;

                ctx.resp.headers.insert("Content-Length", html.len().into());
                ctx.resp.headers.insert("Content-Type", mime::TEXT_HTML_UTF_8.as_ref().parse()?);

                ctx.resp.write(html);

                return Err(status!(http::StatusCode::OK));
            }
        }
    }

    next.await
}