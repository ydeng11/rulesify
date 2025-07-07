use crate::converters::RuleConverter;
use crate::models::rule::{FileReference, RuleCondition, RuleContent, RuleMetadata, UniversalRule};
use anyhow::{anyhow, Result};
use serde_yaml;
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

        // For "Apply Intelligently" mode to work, Cursor needs the description field
        // to contain the actual rule description (not just the name)
        if let Some(desc) = &rule.metadata.description {
            // Use the actual description for intelligent application
            if desc.contains('\n') {
                output.push_str("description: |\n");
                for line in desc.lines() {
                    output.push_str(&format!("  {}\n", line));
                }
            } else {
                output.push_str(&format!("description: \"{}\"\n", desc));
            }
            // Include rule name as notes for reference
            output.push_str(&format!("notes: \"Rule: {}\"\n", rule.metadata.name));
        } else {
            // Fallback to rule name if no description available
            output.push_str(&format!("description: \"{}\"\n", rule.metadata.name));
        }

        // Extract application mode from cursor tool overrides
        let cursor_overrides = rule.tool_overrides.get("cursor");

        // First check for new apply_mode field
        let apply_mode = cursor_overrides
            .and_then(|overrides| overrides.get("apply_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                // Fall back to legacy auto_apply field for backwards compatibility
                let auto_apply = cursor_overrides
                    .and_then(|overrides| overrides.get("auto_apply"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if auto_apply {
                    "always"
                } else {
                    // If auto_apply is false, check if globs exist to determine mode
                    let has_globs = !rule.conditions.is_empty();
                    if has_globs {
                        "specific_files"
                    } else {
                        "intelligent" // Default for backwards compatibility
                    }
                }
            });

        // Only include globs if apply_mode is "specific_files"
        if apply_mode == "specific_files" && !rule.conditions.is_empty() {
            output.push_str("globs:\n");
            for condition in &rule.conditions {
                if let RuleCondition::FilePattern { value } = condition {
                    output.push_str(&format!("  - \"{}\"\n", value));
                }
            }
        }

        // Map apply_mode to Cursor frontmatter
        let always_apply = match apply_mode {
            "always" => true,
            "intelligent" | "specific_files" | "manual" => false,
            _ => false, // Default to false for unknown modes
        };

        output.push_str(&format!("alwaysApply: {}\n", always_apply));
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

        // Extract name from notes field (if in "Rule: XYZ" format) or fallback to description
        let name = frontmatter
            .get("notes")
            .and_then(|v| v.as_str())
            .and_then(|notes| {
                if notes.starts_with("Rule: ") {
                    Some(notes.strip_prefix("Rule: ").unwrap_or(notes).to_string())
                } else {
                    None
                }
            })
            .or_else(|| {
                // Fallback: use description if notes doesn't contain rule name
                frontmatter
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| {
                // Last resort: use first content section title
                content_sections
                    .first()
                    .map(|section| section.title.clone())
            })
            .unwrap_or_else(|| "Imported Rule".to_string());

        // Extract description from description field (new behavior)
        let description = frontmatter
            .get("description")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != name) // Don't use description if it's the same as name
            .map(|s| s.to_string());

        // Extract metadata from frontmatter
        let metadata = RuleMetadata {
            name,
            description,
            tags: Vec::new(),
            priority: 5,
        };

        // Parse conditions from globs
        let conditions: Vec<RuleCondition> = frontmatter
            .get("globs")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| RuleCondition::FilePattern {
                        value: s.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Generate rule ID from name
        let rule_id = metadata
            .name
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        // Create tool overrides with apply_mode for cursor
        let always_apply = frontmatter
            .get("alwaysApply")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Determine apply_mode based on Cursor frontmatter
        let apply_mode = if always_apply {
            "always"
        } else {
            // If alwaysApply is false, check if globs exist to determine mode
            let has_globs = !conditions.is_empty();
            if has_globs {
                "specific_files"
            } else {
                "intelligent" // Default when alwaysApply is false and no globs
            }
        };

        let mut tool_overrides: std::collections::HashMap<String, serde_json::Value> =
            std::collections::HashMap::new();
        let mut cursor_overrides = serde_json::Map::new();

        // Add the new apply_mode field
        cursor_overrides.insert(
            "apply_mode".to_string(),
            serde_json::Value::String(apply_mode.to_string()),
        );

        // Keep auto_apply for backwards compatibility (deprecated)
        cursor_overrides.insert(
            "auto_apply".to_string(),
            serde_json::Value::Bool(always_apply),
        );

        tool_overrides.insert(
            "cursor".to_string(),
            serde_json::Value::Object(cursor_overrides),
        );

        Ok(UniversalRule {
            id: rule_id,
            version: "0.1.0".to_string(),
            metadata,
            content: content_sections,
            references,
            conditions,
            tool_overrides,
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
