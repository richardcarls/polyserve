use std::path::PathBuf;

use roa::{Context, Next, Result};

use crate::{Resource, ServerConfig};

pub async fn resolve_file<S>(ctx: &mut Context<S>, next: Next<'_>) -> Result {
    let res = ctx.load::<Resource>("res");

    if let Some(res) = res.as_deref() {
        let ServerConfig { ref allow_extension_elision, .. } = res.config().server;

        // Search for file with supported extensions
        if allow_extension_elision.len() > 0 {
            let mut candidates = allow_extension_elision
                .iter()
                .map(|ext| {
                    // TODO: Work with Resources instead
                    let mut file_path = res.fs_path().to_owned();

                    if let Some(file_name) = res.fs_path().file_name() {
                        let mut file_name = file_name.to_owned();
                        file_name.push(format!(".{}", ext));

                        file_path.set_file_name(file_name.as_os_str());
                    }

                    file_path
                })
                .filter_map(|path| {
                    if path.is_file() {
                        // TODO: return Resources
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect::<Vec<PathBuf>>();
            
            if candidates.len() > 0 {
                candidates.sort();

                let file_path = candidates.remove(0);
                let file_name = file_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy();
                
                // TODO: Move into Resource resolve
                let mut uri_path = ctx.uri().path().to_owned();
                let boundary = uri_path.rfind('/').unwrap();

                uri_path.truncate(boundary);

                let location = format!("{}/{}", uri_path, file_name);

                ctx.resp.headers.insert("Content-Location", location.parse()?);

                let res = Resource::new(location.as_str(), res.root_path());

                ctx.store("res", res);

                return Ok(())
            }
        }
    }

    next.await
}