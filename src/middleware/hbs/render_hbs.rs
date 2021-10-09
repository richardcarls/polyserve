use std::sync::Arc;

use async_std::fs;
use mime;
use roa::{Context, Next, Result, status, http};
use handlebars::Handlebars;

use crate::{Resource, ServerConfig};
use super::{url_encode_helper, url_decode_helper};

const INDEX_TEMPLATE: &'static str = include_str!("../../../include/templates/index.html.hbs");
const LAYOUT_TEMPLATE: &'static str = include_str!("../../../include/templates/layout.html.hbs");

pub async fn render_hbs<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let mut hbs = Handlebars::new();

    hbs.register_helper("url_encode", Box::new(url_encode_helper));
    hbs.register_helper("url_decode", Box::new(url_decode_helper));

    let res = ctx.load::<Resource>("res");

    if let Some(_res) = res.as_deref() {
        //let ServerConfig { ref index_template, ref layout_template .. } = res.config().server;

        // TODO: look for files based on convention
        // TODO: Template inheritance
        let _ = hbs.register_template_string("index", INDEX_TEMPLATE);
        let _ = hbs.register_partial("layout", LAYOUT_TEMPLATE);
    } else {
        let _ = hbs.register_template_string("index", INDEX_TEMPLATE);
        let _ = hbs.register_partial("layout", LAYOUT_TEMPLATE);
    }

    let hbs = Arc::new(hbs);

    ctx.store("hbs", Arc::clone(&hbs));

    next.await?;
    
    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        let ServerConfig { ref render_hbs, .. } = res.config().server;

        if *render_hbs {
            if let Some(ext) = res.fs_path().extension().as_deref() {
                if ext == "hbs" {
                    let tpl = fs::read_to_string(res.fs_path()).await?;
    
                    let html = hbs.render_template(tpl.as_str(), res.context())?;
    
                    ctx.resp.headers.insert("Content-Length", html.len().into());
                    ctx.resp.headers.insert("Content-Type", mime::TEXT_HTML_UTF_8.as_ref().parse()?);
    
                    ctx.resp.write(html);

                    return Err(status!(http::StatusCode::OK));
                }
            }
        }
    }

    Ok(())
}