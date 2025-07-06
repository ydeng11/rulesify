use crate::converters::RuleConverter;
use crate::models::rule::{UniversalRule, RuleMetadata, RuleContent};
use anyhow::Result;
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

    fn convert_from_tool_format(&self, content: &str) -> Result<UniversalRule> {
        let (name, description, content_sections) = parse_cline_format(content)?;

        // Generate rule ID from name
        let rule_id = name.to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        let metadata = RuleMetadata {
            name,
            description,
            tags: Vec::new(),
            priority: 5,
            auto_apply: false,
        };

        Ok(UniversalRule {
            id: rule_id,
            version: "0.1.0".to_string(),
            metadata,
            content: content_sections,
            references: Vec::new(),
            conditions: Vec::new(),
            tool_overrides: std::collections::HashMap::new(),
        })
    }

    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".clinerules")
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }
}

fn parse_cline_format(content: &str) -> Result<(String, Option<String>, Vec<RuleContent>)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut name = "Imported Rule".to_string();
    let mut description = None;
    let mut content_sections = Vec::new();
    let mut current_section: Option<(String, Vec<String>)> = None;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("# ") {
            // Main title
            name = line[2..].trim().to_string();

            // Check if next non-empty line is description (not a heading)
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }

            if j < lines.len() {
                let next_line = lines[j].trim();
                if !next_line.starts_with('#') && !next_line.is_empty() {
                    // This is likely a description
                    description = Some(next_line.to_string());
                    i = j;
                }
            }
        } else if line.starts_with("## ") {
            // Save previous section if exists
            if let Some((title, content_lines)) = current_section.take() {
                content_sections.push(RuleContent {
                    title,
                    format: crate::models::rule::ContentFormat::Markdown,
                    value: content_lines.join("\n").trim().to_string(),
                });
            }

            // Start new section
            let title = line[3..].trim().to_string();
            current_section = Some((title, Vec::new()));
        } else if let Some((_, ref mut content_lines)) = current_section {
            content_lines.push(line.to_string());
        } else if !line.is_empty() && !line.starts_with('#') {
            // Content without a section header
            if content_sections.is_empty() && current_section.is_none() {
                current_section = Some(("Content".to_string(), vec![line.to_string()]));
            }
        }

        i += 1;
    }

    // Save last section if exists
    if let Some((title, content_lines)) = current_section {
        content_sections.push(RuleContent {
            title,
            format: crate::models::rule::ContentFormat::Markdown,
            value: content_lines.join("\n").trim().to_string(),
        });
    }

    // If no sections found, create a default one with all content
    if content_sections.is_empty() && !content.trim().is_empty() {
        content_sections.push(RuleContent {
            title: "Content".to_string(),
            format: crate::models::rule::ContentFormat::Markdown,
            value: content.trim().to_string(),
        });
    }

    Ok((name, description, content_sections))
}
