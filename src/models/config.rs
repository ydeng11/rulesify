use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub rules_directory: PathBuf,
    pub editor: Option<String>,
    pub default_tools: Vec<String>,
} 