use std::path::PathBuf;

use roa::{Context, Next, Result};

use crate::{Resource, ServerConfig};

pub async fn use_index<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        let ServerConfig { ref use_index, ref allow_extension_elision, .. } = res.config().server;

        if *use_index == false {
            return next.await;
        }

        // Search for index file with supported extensions
        if res.fs_path().is_dir() && allow_extension_elision.len() > 0 {
            let mut candidates = allow_extension_elision
                .iter()
                .map(|ext| res.fs_path().to_owned().join(format!("index.{}", ext)))
                .filter_map(|path| {
                    if path.is_file() {
                        // TODO: Wrap in newtype implementing Comparison
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect::<Vec<PathBuf>>();
            
            if candidates.len() > 0 {
                candidates.sort();

                let index_path = candidates.remove(0);
                let file_name = index_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy();

                let location = format!("{}/{}", ctx.uri().path(), file_name);

                ctx.resp.headers.insert("Content-Location", location.parse()?);
                
                let res = Resource::new(location.as_str(), res.root_path());

                ctx.store("res", res);

                return Ok(())
            }
        }
    }

    next.await
}