use roa::{Context, Result, Next};

use crate::{PolyState, Resource};

pub async fn resolve_resource(ctx: &mut Context<PolyState>, next: Next<'_>) -> Result {
    let res = Resource::new(ctx.uri().path(), ctx.root_path());

    ctx.store("res", res);

    next.await

    /*
    let result = next.await;

    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        log::info!("{:#?}", res);
    }

    result
    */
}