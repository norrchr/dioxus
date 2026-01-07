use crate::icons::IconRegistry;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Configuration for the `dioxus icon` commands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct IconConfig {
    /// The default registry to use
    #[serde(deserialize_with = "empty_string_is_none")]
    pub(crate) default: Option<String>,

    /// Icon registries
    pub(crate) registeries: HashMap<String, IconRegistry>,

    /// The path where icons are stored when adding or removing icons
    #[serde(default)]
    pub(crate) icons_dir: Option<PathBuf>,
}

fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Ok(Some(String::from(s))),
        None => Ok(None),
    }
}