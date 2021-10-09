use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::RequestConfig;

pub const UNKNOWN: &'static str = "(unknown)";

#[derive(Debug)]
pub struct Resource {
    root_path: PathBuf,
    uri_path: String,
    fs_path: PathBuf,
    ancestors: Vec<PathBuf>,
    config: RequestConfig,
    context: ResourceContext,
}

impl Resource {
    pub fn root_path(&self) -> &Path {
        self.root_path.as_path()
    }

    pub fn fs_path(&self) -> &Path {
        self.fs_path.as_path()
    }

    pub fn ancestors(&self) -> Vec<&Path> {
        self.ancestors
            .iter()
            .map(|path| path.as_ref())
            .collect()
    }

    pub fn config(&self) -> &RequestConfig {
        &self.config
    }

    pub fn context(&self) -> &ResourceContext {
        &self.context
    }

    pub fn new(uri_path: &str, root_path: &Path) -> Self {
        let root_path = root_path.to_owned();
        let uri_path = uri_path.to_owned();

        // FS Path
        let fs_path = root_path.to_owned();
        let fs_path = urlencoding::decode(uri_path.as_str())
            .unwrap()
            .split("/")
            .skip(1)
            .fold(fs_path, |path, segment| path.join(segment));
        
        // Ancestors
        let ancestors: Vec<&Path> = fs_path
            .ancestors()
            .filter(|&path| path.starts_with(root_path.as_path()))
            .collect();
        
        // Request Config
        let config = RequestConfig::generate_from_ancestors(&ancestors);

        let context = {
            let name = if let Some(name) = fs_path.file_name() {
                name.to_owned().into_string().ok()
            } else {
                None
            };
    
            let metadata = match fs_path.metadata() {
                Ok(ref meta) => Some(ResourceMetadata::from_meta(meta)),
                Err(_) => None,
            };
    
            let ancestors: Vec<ResourceContext> = ancestors
                .iter()
                .map(|path| ResourceContext::from_path(path, true))
                .collect();
            
            let children = match fs_path.is_dir() {
                true => {
                    if let Ok (read_dir) = fs_path.read_dir() {
                        Some(read_dir
                            .filter_map(|entry| entry.ok())
                            .map(|ref entry| ResourceContext::from_dir_entry(entry))
                            .collect())
                    } else {
                        None
                    }
                },
                false => None
            };
    
            ResourceContext {
                name,
                metadata,
                children, 
    
                ancestors: Some(ancestors),
            }
        };

        let ancestors: Vec<PathBuf> = ancestors.iter().map(|&path| path.to_owned()).collect();
        
        Self {
            root_path,
            uri_path,
            fs_path,
            ancestors,
            config,
            context,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceContext {
    name: Option<String>,
    metadata: Option<ResourceMetadata>,
    children: Option<Vec<Self>>,

    ancestors: Option<Vec<Self>>,
}

impl ResourceContext {
    pub fn from_path(path: &Path, shallow: bool) -> Self {
        let name = if let Some(name) = path.file_name() {
            name.to_owned().into_string().ok()
        } else {
            None
        };

        let metadata = match path.metadata() {
            Ok(ref meta) => Some(ResourceMetadata::from_meta(meta)),
            Err(_) => None,
        };

        if shallow {
            Self {
                name,
                metadata,
                ancestors: None,
                children: None, 
            }
        } else {
            let children = match path.is_dir() {
                true => {
                    if let Ok (read_dir) = path.read_dir() {
                        Some(read_dir
                            .filter_map(|entry| entry.ok())
                            .map(|ref entry| Self::from_dir_entry(entry))
                            .collect())
                    } else {
                        None
                    }
                },
                false => None
            };

            Self {
                name,
                metadata,
                ancestors: None,
                children, 
            }
        }
    }

    pub fn from_dir_entry(entry: &fs::DirEntry) -> Self {
        let name = entry.file_name().into_string().ok();
        
        let metadata = match entry.metadata() {
            Ok(ref meta) => Some(ResourceMetadata::from_meta(meta)),
            Err(_) => None,
        };

        Self {
            name,
            metadata,
            ancestors: None,
            children: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    is_dir: bool,
    is_file: bool,
    len: u64,
    readonly: bool,
    modified: String,
    accessed: String,
    created: String,
}

impl ResourceMetadata {
    pub fn from_meta(meta: &fs::Metadata) -> Self {
        let modified = match meta.modified() {
            Ok(st) => {
                let dt: DateTime<Utc> = st.clone().into();

                format!("{}", dt.format("%+"))
            },
            _ => UNKNOWN.to_owned()
        };

        let accessed = match meta.accessed() {
            Ok(st) => {
                let dt: DateTime<Utc> = st.clone().into();

                format!("{}", dt.format("%+"))
            },
            _ => UNKNOWN.to_owned()
        };

        let created = match meta.created() {
            Ok(st) => {
                let dt: DateTime<Utc> = st.clone().into();

                format!("{}", dt.format("%+"))
            },
            _ => UNKNOWN.to_owned()
        };

        Self {
            is_dir: meta.is_dir(),
            is_file: meta.is_file(),
            len: meta.len(),
            readonly: meta.permissions().readonly(),
            modified,
            accessed,
            created,
        }
    }
}