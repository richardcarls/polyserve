use roa::{Context, Next, Result};

pub async fn server_header<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let Context { resp, .. } = ctx;

    resp.headers.insert("Server", "polyserve 0.1.0".parse()?);

    next.await
}