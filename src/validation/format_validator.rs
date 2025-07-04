use crate::models::rule::UniversalRule;
use crate::validation::{ValidationError, Validator, Severity};
use anyhow::Result;

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

        // Check ID format
        if rule.id.contains(' ') || rule.id.chars().any(|c| c.is_uppercase()) {
            errors.push(ValidationError {
                field: "id".to_string(),
                message: "ID should be lowercase with no spaces (use hyphens or underscores)".to_string(),
                severity: Severity::Warning,
            });
        }

        // TODO: Add more format validation rules

        Ok(errors)
    }
} 