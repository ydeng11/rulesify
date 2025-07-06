use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::store::{RuleStore, file_store::FileStore};
use crate::utils::config::load_config_from_path;
use crate::validation::{
    Validator,
    Severity,
    content_validator::ContentValidator,
    format_validator::FormatValidator,
};

pub fn run(rule: Option<String>, all: bool, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Determine which rules to validate
    let rule_names = if all {
        store.list_rules()?
    } else if let Some(rule_name) = rule {
        vec![rule_name]
    } else {
        anyhow::bail!("Must specify either a rule name or --all");
    };

    if rule_names.is_empty() {
        println!("No rules found to validate");
        return Ok(());
    }

    // Initialize validators
    let validators: Vec<Box<dyn Validator>> = vec![
        Box::new(ContentValidator::new()),
        Box::new(FormatValidator::new()),
    ];

    println!("🔍 Validating {} rule(s)", rule_names.len());
    println!("{}", "─".repeat(50));

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_info = 0;

    for rule_name in &rule_names {
        match validate_rule(&store, &validators, rule_name) {
            Ok(ValidationResult { errors, warnings, info }) => {
                if errors == 0 && warnings == 0 && info == 0 {
                    println!("✅ {}: No issues found", rule_name);
                } else {
                    println!("📋 {}: {} error(s), {} warning(s), {} info",
                             rule_name, errors, warnings, info);
                }
                total_errors += errors;
                total_warnings += warnings;
                total_info += info;
            }
            Err(e) => {
                println!("❌ {}: Failed to validate - {}", rule_name, e);
                total_errors += 1;
            }
        }
    }

    println!("{}", "─".repeat(50));
    println!("📊 Summary: {} error(s), {} warning(s), {} info",
             total_errors, total_warnings, total_info);

    if total_errors > 0 {
        println!("❌ Validation failed with {} error(s)", total_errors);
        std::process::exit(1);
    } else if total_warnings > 0 {
        println!("⚠️  Validation passed with {} warning(s)", total_warnings);
    } else {
        println!("✅ All rules passed validation");
    }

    Ok(())
}

struct ValidationResult {
    errors: usize,
    warnings: usize,
    info: usize,
}

fn validate_rule(
    store: &FileStore,
    validators: &[Box<dyn Validator>],
    rule_name: &str,
) -> Result<ValidationResult> {
    // Load the rule
    let rule = store.load_rule(rule_name)?
        .ok_or_else(|| anyhow::anyhow!("Rule '{}' not found", rule_name))?;

    let mut all_errors = Vec::new();

    // Run all validators
    for validator in validators {
        let errors = validator.validate(&rule)
            .with_context(|| format!("Validator failed for rule '{}'", rule_name))?;
        all_errors.extend(errors);
    }

    // Display validation results
    let mut errors = 0;
    let mut warnings = 0;
    let mut info = 0;

    for error in &all_errors {
        match error.severity {
            Severity::Error => {
                println!("  ❌ {}: {}", error.field, error.message);
                errors += 1;
            }
            Severity::Warning => {
                println!("  ⚠️  {}: {}", error.field, error.message);
                warnings += 1;
            }
            Severity::Info => {
                println!("  ℹ️  {}: {}", error.field, error.message);
                info += 1;
            }
        }
    }

    Ok(ValidationResult { errors, warnings, info })
}
