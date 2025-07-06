use crate::models::rule::UniversalRule;
use crate::validation::{ValidationError, Validator, Severity};
use anyhow::Result;
use regex;

pub struct ContentValidator;

impl ContentValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContentValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for ContentValidator {
    fn validate(&self, rule: &UniversalRule) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check if rule has content
        if rule.content.is_empty() {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Rule must have at least one content section".to_string(),
                severity: Severity::Error,
            });
        }

        // Check if rule has a name
        if rule.metadata.name.trim().is_empty() {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Rule must have a name".to_string(),
                severity: Severity::Error,
            });
        }

        // Check name length
        if rule.metadata.name.len() > 100 {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Rule name should be 100 characters or less".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check if name contains only valid characters
        if rule.metadata.name.chars().any(|c| c.is_control() || c == '\n' || c == '\r') {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Rule name should not contain control characters or newlines".to_string(),
                severity: Severity::Error,
            });
        }

        // Check description length
        if let Some(description) = &rule.metadata.description {
            if description.len() > 500 {
                errors.push(ValidationError {
                    field: "metadata.description".to_string(),
                    message: "Rule description should be 500 characters or less".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check content sections
        for (i, section) in rule.content.iter().enumerate() {
            let field_prefix = format!("content[{}]", i);

            // Check section title
            if section.title.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("{}.title", field_prefix),
                    message: "Content section must have a title".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check section content
            if section.value.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("{}.value", field_prefix),
                    message: "Content section must have content".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for very long content sections
            if section.value.len() > 10000 {
                errors.push(ValidationError {
                    field: format!("{}.value", field_prefix),
                    message: "Content section is very long (>10k chars). Consider breaking it up".to_string(),
                    severity: Severity::Info,
                });
            }

            // Check for duplicate section titles
            for (j, other_section) in rule.content.iter().enumerate() {
                if i != j && section.title == other_section.title {
                    errors.push(ValidationError {
                        field: format!("{}.title", field_prefix),
                        message: format!("Duplicate section title '{}' found", section.title),
                        severity: Severity::Warning,
                    });
                    break;
                }
            }
        }

        // Check priority range
        if rule.metadata.priority > 10 {
            errors.push(ValidationError {
                field: "metadata.priority".to_string(),
                message: "Priority should be between 1 and 10".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check if tags are reasonable
        if rule.metadata.tags.len() > 10 {
            errors.push(ValidationError {
                field: "metadata.tags".to_string(),
                message: "Consider limiting tags to 10 or fewer for better organization".to_string(),
                severity: Severity::Info,
            });
        }

        // Check for empty tags
        for (i, tag) in rule.metadata.tags.iter().enumerate() {
            if tag.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("metadata.tags[{}]", i),
                    message: "Tag cannot be empty".to_string(),
                    severity: Severity::Error,
                });
            }
        }

        // Check file references
        for (i, reference) in rule.references.iter().enumerate() {
            if reference.path.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("references[{}].path", i),
                    message: "File reference path cannot be empty".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for suspicious file paths
            if reference.path.contains("..") {
                errors.push(ValidationError {
                    field: format!("references[{}].path", i),
                    message: "File reference contains '..' which might be unsafe".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check conditions
        for (i, condition) in rule.conditions.iter().enumerate() {
            match condition {
                crate::models::rule::RuleCondition::FilePattern { value } => {
                    if value.trim().is_empty() {
                        errors.push(ValidationError {
                            field: format!("conditions[{}].value", i),
                            message: "File pattern cannot be empty".to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
                crate::models::rule::RuleCondition::Regex { value } => {
                    if value.trim().is_empty() {
                        errors.push(ValidationError {
                            field: format!("conditions[{}].value", i),
                            message: "Regex pattern cannot be empty".to_string(),
                            severity: Severity::Error,
                        });
                    }

                    // Try to compile the regex to check if it's valid
                    if let Err(_) = regex::Regex::new(value) {
                        errors.push(ValidationError {
                            field: format!("conditions[{}].value", i),
                            message: "Invalid regex pattern".to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        // Suggest adding description if missing
        if rule.metadata.description.is_none() {
            errors.push(ValidationError {
                field: "metadata.description".to_string(),
                message: "Consider adding a description for better rule documentation".to_string(),
                severity: Severity::Info,
            });
        }

        // Suggest adding tags if missing
        if rule.metadata.tags.is_empty() {
            errors.push(ValidationError {
                field: "metadata.tags".to_string(),
                message: "Consider adding tags to help categorize and find this rule".to_string(),
                severity: Severity::Info,
            });
        }

        Ok(errors)
    }
}
