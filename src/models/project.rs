use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub rules_directory: PathBuf,
    pub enabled_tools: Vec<String>,
    pub default_template: Option<String>,
} 