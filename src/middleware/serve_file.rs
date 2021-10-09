use async_std::fs::File;
use roa::{Context, Next, Result, status, http};
use mime;
use mime_guess;

use crate::Resource;

pub async fn serve_file<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    next.await?;

    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        if ctx.method() == &http::Method::GET && res.fs_path().is_file() {
            let mime_type = mime_guess::from_path(res.fs_path())
                .first()
                .unwrap_or(mime::TEXT_PLAIN_UTF_8);
            
            let file = File::open(res.fs_path()).await?;
            let metadata = file.metadata().await?;
            
            ctx.resp.headers.insert("Content-Length", metadata.len().into());
            ctx.resp.headers.insert("Content-Type", mime_type.as_ref().parse()?);
                
            ctx.resp.write_reader(file);

            return Err(status!(http::StatusCode::OK));
        }
    }

    ctx.resp.headers.insert("Content-Type", mime::TEXT_PLAIN_UTF_8.as_ref().parse()?);

    Err(status!(http::StatusCode::NOT_FOUND))
}