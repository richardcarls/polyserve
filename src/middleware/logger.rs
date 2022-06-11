use roa::{Context, Next, Result};

pub async fn logger<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    log::info!("--> {} {}", ctx.method(), ctx.uri().path());

    let result = next.await;
    match result {
        Err(ref status) if status.status_code.is_server_error() => {
            log::error!(
                "<-- {} {} {}",
                ctx.method(),
                ctx.uri().path(),
                status.status_code,
            );
        },

        Err(ref status) => {
            log::warn!(
                "<-- {} {} {}",
                ctx.method(),
                ctx.uri().path(),
                status.status_code,
            );
        },

        _ => (),
    }

    result
}