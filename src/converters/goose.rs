use crate::converters::RuleConverter;
use crate::models::rule::{RuleContent, RuleMetadata, UniversalRule};
use crate::utils::rule_id::{determine_rule_id_with_fallback, embed_rule_id_in_content};
use anyhow::Result;
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

        // Embed rule ID for tracking
        let output_with_id = embed_rule_id_in_content(&output, &rule.id);

        Ok(output_with_id)
    }

    fn convert_from_tool_format(&self, content: &str) -> Result<UniversalRule> {
        let (name, description, content_sections) = parse_goose_format(content)?;

        // Generate rule ID using fallback hierarchy
        let rule_id = determine_rule_id_with_fallback(
            content,
            None, // No filename context in convert_from_tool_format
            Some(&name),
        )?;

        let metadata = RuleMetadata {
            name,
            description,
            tags: Vec::new(),
            priority: 5,
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
        project_root.to_path_buf()
    }

    fn get_file_extension(&self) -> &str {
        "goosehints"
    }
}

fn parse_goose_format(content: &str) -> Result<(String, Option<String>, Vec<RuleContent>)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut name = "Imported Rule".to_string();
    let mut description = None;
    let mut content_sections = Vec::new();
    let mut current_section: Option<(String, Vec<String>)> = None;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if i + 1 < lines.len() && !line.is_empty() {
            let next_line = lines[i + 1].trim();

            // Check if this is a title with underline (= or -)
            if (next_line.chars().all(|c| c == '=') && next_line.len() > 0)
                || (next_line.chars().all(|c| c == '-') && next_line.len() > 0)
            {
                if next_line.chars().all(|c| c == '=') {
                    // Main title (underlined with =)
                    name = line.to_string();
                    i += 1; // Skip the underline

                    // Check if next non-empty line is description (not underlined)
                    let mut j = i + 1;
                    while j < lines.len() && lines[j].trim().is_empty() {
                        j += 1;
                    }

                    if j < lines.len() {
                        let desc_line = lines[j].trim();
                        if !desc_line.is_empty() {
                            // Check if the line after description is not an underline
                            let is_description = if j + 1 < lines.len() {
                                let after_desc = lines[j + 1].trim();
                                !(after_desc.chars().all(|c| c == '=' || c == '-')
                                    && after_desc.len() > 0)
                            } else {
                                true
                            };

                            if is_description {
                                description = Some(desc_line.to_string());
                                i = j;
                            }
                        }
                    }
                } else {
                    // Section title (underlined with -)
                    // Save previous section if exists
                    if let Some((title, content_lines)) = current_section.take() {
                        let content_value = content_lines.join("\n").trim().to_string();
                        if !content_value.is_empty() {
                            content_sections.push(RuleContent {
                                title,
                                format: crate::models::rule::ContentFormat::PlainText,
                                value: content_value,
                            });
                        }
                    }

                    // Start new section
                    current_section = Some((line.to_string(), Vec::new()));
                    i += 1; // Skip the underline
                }
            } else if let Some((_, ref mut content_lines)) = current_section {
                // Skip rulesify HTML comments
                if !line.starts_with("<!-- rulesify-id:") {
                    content_lines.push(line.to_string());
                }
            } else if !line.is_empty() && !line.starts_with("<!-- rulesify-id:") {
                // Content without a section header after we've found the main title (skip rulesify HTML comments)
                if !name.is_empty() && name != "Imported Rule" {
                    if current_section.is_none() {
                        current_section = Some(("Content".to_string(), vec![line.to_string()]));
                    }
                }
            }
        } else if let Some((_, ref mut content_lines)) = current_section {
            content_lines.push(line.to_string());
        } else if !line.is_empty() {
            // Content without a section header
            if current_section.is_none() && (name.is_empty() || name == "Imported Rule") {
                // If we haven't found a title yet, this might be content
                current_section = Some(("Content".to_string(), vec![line.to_string()]));
            }
        }

        i += 1;
    }

    // Save last section if exists
    if let Some((title, content_lines)) = current_section {
        let content_value = content_lines.join("\n").trim().to_string();
        if !content_value.is_empty() {
            content_sections.push(RuleContent {
                title,
                format: crate::models::rule::ContentFormat::PlainText,
                value: content_value,
            });
        }
    }

    // If no sections found, create a default one with all content
    if content_sections.is_empty() && !content.trim().is_empty() {
        content_sections.push(RuleContent {
            title: "Content".to_string(),
            format: crate::models::rule::ContentFormat::PlainText,
            value: content.trim().to_string(),
        });
    }

    Ok((name, description, content_sections))
}
