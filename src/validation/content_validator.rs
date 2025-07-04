use crate::models::rule::UniversalRule;
use crate::validation::{ValidationError, Validator, Severity};
use anyhow::Result;

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

        // TODO: Add more content validation rules

        Ok(errors)
    }
} 