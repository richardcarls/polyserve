use roa::{Context, Next, Result};

pub async fn early_return<S>(_: &mut Context<S>, next: Next<'_>) -> Result {
    match next.await {
        Err(status) if status.status_code.is_client_error() => Err(status),
        Err(status) if status.status_code.is_server_error() => Err(status),
        Err(status) if status.status_code.is_redirection() => Err(status),
        _ => Ok(())
    }
}