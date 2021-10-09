use roa::{Context, Next, Result};

pub async fn early_return<S>(_: &mut Context<S>, next: Next<'_>) -> Result {
    match next.await {
        Ok(()) => Ok(()),
        Err(status) => {
            if status.status_code.is_success() {
                Ok(())
            } else {
                Err(status)
            }
        },
    }
}