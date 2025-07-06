use std::collections::HashMap;
use rulesify::models::rule::{UniversalRule, RuleMetadata, RuleContent, ContentFormat, RuleCondition, FileReference};
use rulesify::validation::{Validator, Severity, content_validator::ContentValidator, format_validator::FormatValidator};

#[test]
fn test_content_validator_valid_rule() {
    let validator = ContentValidator::new();
    let rule = create_valid_rule();

    let errors = validator.validate(&rule).unwrap();
    // Should have some info suggestions but no errors
    let error_count = errors.iter().filter(|e| matches!(e.severity, Severity::Error)).count();
    assert_eq!(error_count, 0);
}

#[test]
fn test_content_validator_missing_name() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.name = "".to_string();

    let errors = validator.validate(&rule).unwrap();
    let error_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| &e.message)
        .collect();

    assert!(error_messages.iter().any(|msg| msg.contains("Rule must have a name")));
}

#[test]
fn test_content_validator_empty_content() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.content = vec![];

    let errors = validator.validate(&rule).unwrap();
    let error_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| &e.message)
        .collect();

    assert!(error_messages.iter().any(|msg| msg.contains("Rule must have at least one content section")));
}

#[test]
fn test_content_validator_empty_section_title() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.content[0].title = "".to_string();

    let errors = validator.validate(&rule).unwrap();
    let error_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| &e.message)
        .collect();

    assert!(error_messages.iter().any(|msg| msg.contains("Content section must have a title")));
}

#[test]
fn test_content_validator_empty_section_content() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.content[0].value = "".to_string();

    let errors = validator.validate(&rule).unwrap();
    let error_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| &e.message)
        .collect();

    assert!(error_messages.iter().any(|msg| msg.contains("Content section must have content")));
}

#[test]
fn test_content_validator_duplicate_section_titles() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.content.push(RuleContent {
        title: rule.content[0].title.clone(), // Duplicate title
        format: ContentFormat::Markdown,
        value: "Different content".to_string(),
    });

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("Duplicate section title")));
}

#[test]
fn test_content_validator_long_name() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.name = "a".repeat(150); // Very long name

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("Rule name should be 100 characters or less")));
}

#[test]
fn test_content_validator_empty_tags() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.tags = vec!["".to_string(), "valid-tag".to_string()];

    let errors = validator.validate(&rule).unwrap();
    let error_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| &e.message)
        .collect();

    assert!(error_messages.iter().any(|msg| msg.contains("Tag cannot be empty")));
}

#[test]
fn test_content_validator_dangerous_file_reference() {
    let validator = ContentValidator::new();
    let mut rule = create_valid_rule();
    rule.references = vec![FileReference {
        path: "../../../etc/passwd".to_string(),
    }];

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("contains '..' which might be unsafe")));
}

#[test]
fn test_format_validator_valid_rule() {
    let validator = FormatValidator::new();
    let rule = create_valid_rule();

    let errors = validator.validate(&rule).unwrap();
    // Should have some info suggestions but no errors
    let error_count = errors.iter().filter(|e| matches!(e.severity, Severity::Error)).count();
    assert_eq!(error_count, 0);
}

#[test]
fn test_format_validator_invalid_version() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.version = "invalid-version".to_string();

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("Version should follow semantic versioning")));
}

#[test]
fn test_format_validator_invalid_id() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.id = "Invalid ID With Spaces".to_string();

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("ID should be lowercase with no spaces")));
}

#[test]
fn test_format_validator_short_id() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.id = "a".to_string();

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("ID should be at least 2 characters long")));
}

#[test]
fn test_format_validator_long_id() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.id = "a".repeat(60); // Very long ID

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("ID should be 50 characters or less")));
}

#[test]
fn test_format_validator_tags_with_spaces() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.tags = vec!["valid-tag".to_string(), "invalid tag".to_string()];

    let errors = validator.validate(&rule).unwrap();
    let info_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Info))
        .map(|e| &e.message)
        .collect();

    assert!(info_messages.iter().any(|msg| msg.contains("Tags should not contain spaces")));
}

#[test]
fn test_format_validator_uppercase_tags() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.tags = vec!["ValidTag".to_string(), "another-tag".to_string()];

    let errors = validator.validate(&rule).unwrap();
    let info_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Info))
        .map(|e| &e.message)
        .collect();

    assert!(info_messages.iter().any(|msg| msg.contains("Tags should be lowercase for consistency")));
}

#[test]
fn test_format_validator_duplicate_tags() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.metadata.tags = vec!["tag1".to_string(), "tag2".to_string(), "tag1".to_string()];

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("Duplicate tag")));
}

#[test]
fn test_format_validator_absolute_file_reference() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.references = vec![FileReference {
        path: "/absolute/path/to/file.md".to_string(),
    }];

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("should use relative paths")));
}

#[test]
fn test_format_validator_windows_path_separators() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.references = vec![FileReference {
        path: "docs\\readme.md".to_string(),
    }];

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("should use forward slashes")));
}

#[test]
fn test_format_validator_broad_file_patterns() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.conditions = vec![RuleCondition::FilePattern {
        value: "**/*".to_string(),
    }];

    let errors = validator.validate(&rule).unwrap();
    let info_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Info))
        .map(|e| &e.message)
        .collect();

    assert!(info_messages.iter().any(|msg| msg.contains("File pattern is very broad")));
}

#[test]
fn test_format_validator_yaml_in_content() {
    let validator = FormatValidator::new();
    let mut rule = create_valid_rule();
    rule.content[0].value = "---\nkey: value\n---\nThis looks like YAML".to_string();

    let errors = validator.validate(&rule).unwrap();
    let warning_messages: Vec<_> = errors.iter()
        .filter(|e| matches!(e.severity, Severity::Warning))
        .map(|e| &e.message)
        .collect();

    assert!(warning_messages.iter().any(|msg| msg.contains("appears to contain YAML syntax")));
}

#[test]
fn test_validation_multiple_validators() {
    let content_validator = ContentValidator::new();
    let format_validator = FormatValidator::new();

    // Create a rule with multiple issues
    let mut rule = create_valid_rule();
    rule.metadata.name = "".to_string(); // Content validator error
    rule.id = "Invalid ID".to_string(); // Format validator warning
    rule.content = vec![]; // Content validator error

    let content_errors = content_validator.validate(&rule).unwrap();
    let format_errors = format_validator.validate(&rule).unwrap();

    let total_errors = content_errors.iter().chain(format_errors.iter())
        .filter(|e| matches!(e.severity, Severity::Error))
        .count();

    assert!(total_errors >= 2); // At least 2 errors from both validators
}

fn create_valid_rule() -> UniversalRule {
    UniversalRule {
        id: "test-rule".to_string(),
        version: "1.0.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("A test rule for validation".to_string()),
            tags: vec!["test".to_string(), "validation".to_string()],
            priority: 5,
            auto_apply: false,
        },
        content: vec![
            RuleContent {
                title: "Guidelines".to_string(),
                format: ContentFormat::Markdown,
                value: "Follow these guidelines:\n- Be consistent\n- Write tests".to_string(),
            },
        ],
        references: vec![
            FileReference {
                path: "README.md".to_string(),
            },
        ],
        conditions: vec![
            RuleCondition::FilePattern {
                value: "src/**/*.ts".to_string(),
            },
        ],
        tool_overrides: HashMap::new(),
    }
}
