pub mod cursor;
pub mod cline;
pub mod claude_code;
pub mod goose;

use crate::models::rule::UniversalRule;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait RuleConverter {
    fn convert_to_tool_format(&self, rule: &UniversalRule) -> Result<String>;
    fn convert_from_tool_format(&self, content: &str) -> Result<UniversalRule>;
    fn get_deployment_path(&self, project_root: &Path) -> PathBuf;
    fn get_file_extension(&self) -> &str;
} 