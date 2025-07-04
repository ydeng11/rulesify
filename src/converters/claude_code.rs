use crate::converters::RuleConverter;
use crate::models::rule::UniversalRule;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub struct ClaudeCodeConverter;

impl ClaudeCodeConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClaudeCodeConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleConverter for ClaudeCodeConverter {
    fn convert_to_tool_format(&self, rule: &UniversalRule) -> Result<String> {
        let mut output = String::new();
        
        // Claude Code uses CLAUDE.md format
        output.push_str(&format!("# {}\n\n", rule.metadata.name));
        
        if let Some(description) = &rule.metadata.description {
            output.push_str(&format!("{}\n\n", description));
        }
        
        for section in &rule.content {
            output.push_str(&format!("## {}\n\n", section.title));
            output.push_str(&section.value);
            output.push_str("\n\n");
        }
        
        Ok(output)
    }
    
    fn convert_from_tool_format(&self, _content: &str) -> Result<UniversalRule> {
        // TODO: Implement parsing from Claude Code format
        Err(anyhow!("Claude Code import not yet implemented"))
    }
    
    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.to_path_buf()
    }
    
    fn get_file_extension(&self) -> &str {
        "md"
    }
} 