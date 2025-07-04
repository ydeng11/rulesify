use crate::converters::RuleConverter;
use crate::models::rule::{UniversalRule, RuleCondition};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub struct CursorConverter;

impl CursorConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CursorConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleConverter for CursorConverter {
    fn convert_to_tool_format(&self, rule: &UniversalRule) -> Result<String> {
        let mut output = String::new();
        
        // Generate YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("description: {}\n", 
            rule.metadata.description.as_deref().unwrap_or(&rule.metadata.name)));
        
        if !rule.conditions.is_empty() {
            output.push_str("globs:\n");
            for condition in &rule.conditions {
                if let RuleCondition::FilePattern { value } = condition {
                    output.push_str(&format!("  - {}\n", value));
                }
            }
        }
        
        output.push_str(&format!("alwaysApply: {}\n", rule.metadata.auto_apply));
        output.push_str("---\n\n");
        
        // Add content sections
        for section in &rule.content {
            output.push_str(&format!("# {}\n\n", section.title));
            output.push_str(&section.value);
            output.push_str("\n\n");
        }
        
        // Add file references
        for reference in &rule.references {
            output.push_str(&format!("@{}\n", reference.path));
        }
        
        Ok(output)
    }
    
    fn convert_from_tool_format(&self, _content: &str) -> Result<UniversalRule> {
        // TODO: Implement parsing from Cursor format
        Err(anyhow!("Cursor import not yet implemented"))
    }
    
    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".cursor/rules")
    }
    
    fn get_file_extension(&self) -> &str {
        "mdc"
    }
} 