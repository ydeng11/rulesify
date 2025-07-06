use crate::converters::RuleConverter;
use crate::models::rule::{UniversalRule, RuleCondition, RuleMetadata, RuleContent, FileReference};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use serde_yaml;

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
        output.push_str(&format!("description: {}\n", rule.metadata.name));

        if let Some(desc) = &rule.metadata.description {
            output.push_str(&format!("notes: {}\n", desc));
        }

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

    fn convert_from_tool_format(&self, content: &str) -> Result<UniversalRule> {
        // Parse YAML frontmatter and Markdown content
        let (frontmatter, markdown) = parse_cursor_format(content)?;

        // Parse content sections and references from markdown
        let (content_sections, references) = parse_markdown_content(&markdown)?;

        // Extract name from frontmatter or first content section
        let name = if let Some(description) = frontmatter.get("description").and_then(|v| v.as_str()) {
            description.to_string()
        } else if let Some(first_section) = content_sections.first() {
            first_section.title.clone()
        } else {
            "Imported Rule".to_string()
        };

        // Extract description from notes field
        let description = frontmatter.get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract metadata from frontmatter
        let metadata = RuleMetadata {
            name,
            description,
            tags: Vec::new(),
            priority: 5,
            auto_apply: frontmatter.get("alwaysApply")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };

        // Parse conditions from globs
        let conditions = frontmatter.get("globs")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| RuleCondition::FilePattern { value: s.to_string() })
                    .collect()
            })
            .unwrap_or_default();

        // Generate rule ID from name
        let rule_id = metadata.name.to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        Ok(UniversalRule {
            id: rule_id,
            version: "0.1.0".to_string(),
            metadata,
            content: content_sections,
            references,
            conditions,
            tool_overrides: std::collections::HashMap::new(),
        })
    }

    fn get_deployment_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".cursor/rules")
    }

    fn get_file_extension(&self) -> &str {
        "mdc"
    }
}

fn parse_cursor_format(content: &str) -> Result<(serde_yaml::Value, String)> {
    // Check if content starts with YAML frontmatter
    if !content.starts_with("---") {
        return Ok((serde_yaml::Value::Null, content.to_string()));
    }

    // Find the end of frontmatter
    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_end = 0;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            frontmatter_end = i;
            break;
        }
    }

    if frontmatter_end == 0 {
        return Err(anyhow!("Invalid YAML frontmatter: missing closing ---"));
    }

    // Parse frontmatter
    let frontmatter_str = lines[1..frontmatter_end].join("\n");
    let frontmatter: serde_yaml::Value = serde_yaml::from_str(&frontmatter_str)
        .map_err(|e| anyhow!("Failed to parse YAML frontmatter: {}", e))?;

    // Extract markdown content
    let markdown_lines = &lines[frontmatter_end + 1..];
    let markdown = markdown_lines.join("\n").trim().to_string();

    Ok((frontmatter, markdown))
}

fn parse_markdown_content(markdown: &str) -> Result<(Vec<RuleContent>, Vec<FileReference>)> {
    let mut content_sections = Vec::new();
    let mut references = Vec::new();

    let lines: Vec<&str> = markdown.lines().collect();
    let mut current_section: Option<(String, Vec<String>)> = None;

    for line in lines {
        if line.starts_with("# ") || line.starts_with("## ") {
            // Save previous section if exists
            if let Some((title, content_lines)) = current_section.take() {
                let content_value = content_lines.join("\n").trim().to_string();
                if !content_value.is_empty() || !title.is_empty() {
                    content_sections.push(RuleContent {
                        title,
                        format: crate::models::rule::ContentFormat::Markdown,
                        value: content_value,
                    });
                }
            }

            // Start new section
            let title = if line.starts_with("# ") {
                line[2..].trim().to_string()
            } else {
                line[3..].trim().to_string()
            };
            current_section = Some((title, Vec::new()));
        } else if line.starts_with("@") {
            // File reference
            let path = line[1..].trim().to_string();
            references.push(FileReference { path });
        } else if let Some((_, ref mut content_lines)) = current_section {
            content_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            // Content before any heading - start a default section
            if current_section.is_none() {
                current_section = Some(("Content".to_string(), vec![line.to_string()]));
            }
        }
    }

    // Save last section if exists
    if let Some((title, content_lines)) = current_section {
        let content_value = content_lines.join("\n").trim().to_string();
        if !content_value.is_empty() || !title.is_empty() {
            content_sections.push(RuleContent {
                title,
                format: crate::models::rule::ContentFormat::Markdown,
                value: content_value,
            });
        }
    }

    // If no sections found, create a default one
    if content_sections.is_empty() && !markdown.trim().is_empty() {
        content_sections.push(RuleContent {
            title: "Content".to_string(),
            format: crate::models::rule::ContentFormat::Markdown,
            value: markdown.trim().to_string(),
        });
    }

    Ok((content_sections, references))
}
