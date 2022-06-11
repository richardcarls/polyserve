use roa::{Context, Next, Result, status, http};

use crate::{Resource, ServerConfig};

pub async fn allow_methods<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let res = ctx.load::<Resource>("res");
    
    if let Some(res) = res.as_deref() {
        let ServerConfig { ref allow_methods, .. } = res.config().server;
        
        if allow_methods.iter().any(|method| method == ctx.method().as_ref()) {
            return next.await;
        }
    }

    Err(status!(http::StatusCode::METHOD_NOT_ALLOWED))
}