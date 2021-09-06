use std::fs;
use std::path::Path;

use futures::AsyncWrite;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{Context, Result};

pub const UNKNOWN: &'static str = "(unknown)";

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceContext {
    name: String,
    metadata: Option<ResourceMetadata>,
    children: Option<Vec<Box<ResourceContext>>>,
}

impl ResourceContext {
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = match path.file_name() {
            Some(name) => name.to_owned().into_string().unwrap_or(UNKNOWN.to_owned()),
            None => UNKNOWN.to_owned()
        };

        let metadata = match path.metadata() {
            Ok(ref meta) => Some(ResourceMetadata::from_meta(meta)),
            Err(_) => None,
        };
        
        // TODO: Ancestors
        
        let children = match path.is_dir() {
            true => {
                Some(path.read_dir()?
                    .filter_map(|entry| entry.ok())
                    .map(|ref entry| Box::new(Self::from_dir_entry(entry)))
                    .collect())
            },
            false => None
        };

        Ok(Self {
            name,
            metadata,
            children, 
        })
    }

    pub fn from_dir_entry(entry: &fs::DirEntry) -> Self {
        let name = entry.file_name().to_owned().into_string()
                    .unwrap_or(UNKNOWN.to_owned());
        
        let metadata = match entry.metadata() {
            Ok(ref meta) => Some(ResourceMetadata::from_meta(meta)),
            Err(_) => None,
        };

        Self {
            name,
            metadata,
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

#[async_trait]
pub trait Respond {
    async fn respond<W>(self, context: &Context, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send;
}