use crate::converters::RuleConverter;
use crate::models::rule::UniversalRule;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub struct GooseConverter;

impl GooseConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GooseConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleConverter for GooseConverter {
    fn convert_to_tool_format(&self, rule: &UniversalRule) -> Result<String> {
        let mut output = String::new();
        
        // Goose uses simple plain text format
        output.push_str(&format!("{}\n", rule.metadata.name));
        output.push_str(&"=".repeat(rule.metadata.name.len()));
        output.push_str("\n\n");
        
        if let Some(description) = &rule.metadata.description {
            output.push_str(&format!("{}\n\n", description));
        }
        
        for section in &rule.content {
            output.push_str(&format!("{}\n", section.title));
            output.push_str(&"-".repeat(section.title.len()));
            output.push_str("\n");
            output.push_str(&section.value);
            output.push_str("\n\n");
        }
        
        Ok(output)
    }
    
    fn convert_from_tool_format(&self, _content: &str) -> Result<UniversalRule> {
        // TODO: Implement parsing from Goose format
        Err(anyhow!("Goose import not yet implemented"))
    }
    
    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.to_path_buf()
    }
    
    fn get_file_extension(&self) -> &str {
        "goosehints"
    }
} 