use crate::converters::RuleConverter;
use crate::models::rule::UniversalRule;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub struct ClineConverter;

impl ClineConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClineConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleConverter for ClineConverter {
    fn convert_to_tool_format(&self, rule: &UniversalRule) -> Result<String> {
        let mut output = String::new();
        
        // Cline uses simple Markdown format
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
        // TODO: Implement parsing from Cline format
        Err(anyhow!("Cline import not yet implemented"))
    }
    
    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".clinerules")
    }
    
    fn get_file_extension(&self) -> &str {
        "md"
    }
} 