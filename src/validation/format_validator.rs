use crate::models::rule::UniversalRule;
use crate::validation::{ValidationError, Validator, Severity};
use anyhow::Result;
use regex::Regex;

pub struct FormatValidator;

impl FormatValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FormatValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for FormatValidator {
    fn validate(&self, rule: &UniversalRule) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check version format
        if !rule.version.contains('.') {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version should follow semantic versioning (e.g., 0.1.0)".to_string(),
                severity: Severity::Warning,
            });
        }

        // Validate semantic version format more strictly
        let semver_regex = Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9\-]+)?(\+[a-zA-Z0-9\-]+)?$")?;
        if !semver_regex.is_match(&rule.version) {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version should follow semantic versioning format (major.minor.patch)".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check ID format
        if rule.id.contains(' ') || rule.id.chars().any(|c| c.is_uppercase()) {
            errors.push(ValidationError {
                field: "id".to_string(),
                message: "ID should be lowercase with no spaces (use hyphens or underscores)".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check ID format more strictly
        let id_regex = Regex::new(r"^[a-z0-9][a-z0-9\-_]*[a-z0-9]$|^[a-z0-9]$")?;
        if !id_regex.is_match(&rule.id) {
            errors.push(ValidationError {
                field: "id".to_string(),
                message: "ID should start and end with alphanumeric characters, use only lowercase letters, numbers, hyphens, and underscores".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check if ID is too long
        if rule.id.len() > 50 {
            errors.push(ValidationError {
                field: "id".to_string(),
                message: "ID should be 50 characters or less".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check if ID is too short
        if rule.id.len() < 2 {
            errors.push(ValidationError {
                field: "id".to_string(),
                message: "ID should be at least 2 characters long".to_string(),
                severity: Severity::Warning,
            });
        }

        // Validate tag format
        for (i, tag) in rule.metadata.tags.iter().enumerate() {
            // Check for special characters in tags
            if tag.contains(' ') {
                errors.push(ValidationError {
                    field: format!("metadata.tags[{}]", i),
                    message: "Tags should not contain spaces (use hyphens or underscores)".to_string(),
                    severity: Severity::Info,
                });
            }

            // Check tag length
            if tag.len() > 30 {
                errors.push(ValidationError {
                    field: format!("metadata.tags[{}]", i),
                    message: "Tags should be 30 characters or less".to_string(),
                    severity: Severity::Warning,
                });
            }

            // Check for uppercase in tags
            if tag.chars().any(|c| c.is_uppercase()) {
                errors.push(ValidationError {
                    field: format!("metadata.tags[{}]", i),
                    message: "Tags should be lowercase for consistency".to_string(),
                    severity: Severity::Info,
                });
            }
        }

        // Check for duplicate tags
        for (i, tag) in rule.metadata.tags.iter().enumerate() {
            for (j, other_tag) in rule.metadata.tags.iter().enumerate() {
                if i != j && tag == other_tag {
                    errors.push(ValidationError {
                        field: format!("metadata.tags[{}]", i),
                        message: format!("Duplicate tag '{}' found", tag),
                        severity: Severity::Warning,
                    });
                    break;
                }
            }
        }

        // Check file reference format
        for (i, reference) in rule.references.iter().enumerate() {
            if reference.path.starts_with('/') {
                errors.push(ValidationError {
                    field: format!("references[{}].path", i),
                    message: "File reference should use relative paths, not absolute paths".to_string(),
                    severity: Severity::Warning,
                });
            }

            // Check for Windows path separators
            if reference.path.contains('\\') {
                errors.push(ValidationError {
                    field: format!("references[{}].path", i),
                    message: "File reference should use forward slashes (/) for cross-platform compatibility".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check content format consistency
        let markdown_sections = rule.content.iter().filter(|s| matches!(s.format, crate::models::rule::ContentFormat::Markdown)).count();
        let plaintext_sections = rule.content.iter().filter(|s| matches!(s.format, crate::models::rule::ContentFormat::PlainText)).count();

        if markdown_sections > 0 && plaintext_sections > 0 {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Mixing Markdown and plaintext sections. Consider using consistent formatting".to_string(),
                severity: Severity::Info,
            });
        }

        // Check for YAML syntax in content (common mistake)
        for (i, section) in rule.content.iter().enumerate() {
            if section.value.lines().any(|line| line.trim().starts_with("---") || line.contains(": ") && line.trim().ends_with(":")) {
                errors.push(ValidationError {
                    field: format!("content[{}].value", i),
                    message: "Content appears to contain YAML syntax. This should be in the URF metadata, not content".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for suspicious file patterns in conditions
        for (i, condition) in rule.conditions.iter().enumerate() {
            match condition {
                crate::models::rule::RuleCondition::FilePattern { value } => {
                    // Check for Windows path patterns
                    if value.contains('\\') {
                        errors.push(ValidationError {
                            field: format!("conditions[{}].value", i),
                            message: "File patterns should use forward slashes (/) for cross-platform compatibility".to_string(),
                            severity: Severity::Warning,
                        });
                    }

                    // Check for overly broad patterns
                    if value == "*" || value == "**" || value == "**/*" {
                        errors.push(ValidationError {
                            field: format!("conditions[{}].value", i),
                            message: "File pattern is very broad and may match unintended files".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
                crate::models::rule::RuleCondition::Regex { value: _ } => {
                    // Regex conditions are handled by content validator
                    // Could add regex-specific format checks here if needed
                }
            }
        }

        // Check for required fields in tool_overrides
        if let Some(cursor_override) = rule.tool_overrides.get("cursor") {
            if cursor_override.is_object() {
                // Could add specific cursor validation here
            }
        }

        Ok(errors)
    }
}
