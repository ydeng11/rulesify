pub mod content_validator;
pub mod format_validator;

use crate::models::rule::UniversalRule;
use anyhow::Result;

pub trait Validator {
    fn validate(&self, rule: &UniversalRule) -> Result<Vec<ValidationError>>;
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
} 