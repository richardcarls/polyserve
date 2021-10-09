use std::path::Path;

use serde::Deserialize;
use config::{Config, File, FileFormat};

const DEFAULT_CONFIG: &'static str = include_str!("../include/default.toml");

// TODO: Remove config-rs dependency
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RequestConfig {
    pub server: ServerConfig,
}

impl RequestConfig {
    pub fn generate_from_ancestors(ancestors: &Vec<&Path>) -> Self {
        let mut cfg = Config::default();

        let _ = cfg.merge(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));

        ancestors
            .iter()
            .map(|path| path.join(".config.toml"))
            .filter(|path| path.is_file())
            .for_each(|config_path| {
                let _ = cfg.merge(File::from(config_path.as_path()));
            });

        let config = cfg.try_into()
            .unwrap_or(Self::default());
        
        config
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServerConfig {
    pub test: String,
    pub force_trailing_slash: bool,
    pub allow_extension_elision: Vec<String>,
    pub use_index: bool,
    pub auto_index: bool,
    pub allow_methods: Vec<String>,
    pub render_hbs: bool,
}