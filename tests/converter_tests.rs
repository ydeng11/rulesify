use rulesify::converters::{
    RuleConverter,
    cursor::CursorConverter,
    cline::ClineConverter,
    claude_code::ClaudeCodeConverter,
    goose::GooseConverter,
};
use rulesify::models::rule::{
    UniversalRule, RuleMetadata, RuleContent, ContentFormat,
    FileReference, RuleCondition
};
use std::collections::HashMap;

fn create_test_rule() -> UniversalRule {
    UniversalRule {
        id: "test-rule".to_string(),
        version: "1.0.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("A test rule for unit testing".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            priority: 5,
            auto_apply: false,
        },
        content: vec![
            RuleContent {
                title: "Guidelines".to_string(),
                format: ContentFormat::Markdown,
                value: "• Follow test conventions\n• Write clear code".to_string(),
            },
            RuleContent {
                title: "Examples".to_string(),
                format: ContentFormat::Code,
                value: "```rust\nfn test() { assert!(true); }\n```".to_string(),
            },
        ],
        references: vec![
            FileReference {
                path: "README.md".to_string(),
            },
        ],
        conditions: vec![
            RuleCondition::FilePattern {
                value: "src/**/*.rs".to_string()
            },
        ],
        tool_overrides: HashMap::new(),
    }
}

#[test]
fn test_cursor_converter_basic_conversion() {
    let converter = CursorConverter::new();
    let rule = create_test_rule();

    let result = converter.convert_to_tool_format(&rule);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check YAML frontmatter
    assert!(output.starts_with("---\n"));
    assert!(output.contains("description: Test Rule"));
    assert!(output.contains("notes: A test rule for unit testing"));
    assert!(output.contains("alwaysApply: false"));
    assert!(output.contains("globs:\n  - src/**/*.rs"));

    // Check content sections
    assert!(output.contains("# Guidelines"));
    assert!(output.contains("• Follow test conventions"));
    assert!(output.contains("# Examples"));
    assert!(output.contains("```rust"));

    // Check file references
    assert!(output.contains("@README.md"));
}

#[test]
fn test_cline_converter_basic_conversion() {
    let converter = ClineConverter::new();
    let rule = create_test_rule();

    let result = converter.convert_to_tool_format(&rule);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check main title
    assert!(output.starts_with("# Test Rule\n\n"));

    // Check description
    assert!(output.contains("A test rule for unit testing"));

    // Check content sections
    assert!(output.contains("## Guidelines"));
    assert!(output.contains("• Follow test conventions"));
    assert!(output.contains("## Examples"));
    assert!(output.contains("```rust"));
}

#[test]
fn test_claude_code_converter_basic_conversion() {
    let converter = ClaudeCodeConverter::new();
    let rule = create_test_rule();

    let result = converter.convert_to_tool_format(&rule);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check main title
    assert!(output.starts_with("# Test Rule\n\n"));

    // Check description
    assert!(output.contains("A test rule for unit testing"));

    // Check content sections (similar to Cline)
    assert!(output.contains("## Guidelines"));
    assert!(output.contains("• Follow test conventions"));
    assert!(output.contains("## Examples"));
    assert!(output.contains("```rust"));
}

#[test]
fn test_goose_converter_basic_conversion() {
    let converter = GooseConverter::new();
    let rule = create_test_rule();

    let result = converter.convert_to_tool_format(&rule);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check title with underline
    assert!(output.starts_with("Test Rule\n========="));

    // Check description
    assert!(output.contains("A test rule for unit testing"));

    // Check content sections with dashes
    assert!(output.contains("Guidelines\n----------"));
    assert!(output.contains("• Follow test conventions"));
    assert!(output.contains("Examples\n--------"));
    assert!(output.contains("```rust"));
}

#[test]
fn test_all_converters_implement_trait() {
    let rule = create_test_rule();

    // Test that all converters implement the trait correctly
    let converters: Vec<Box<dyn RuleConverter>> = vec![
        Box::new(CursorConverter::new()),
        Box::new(ClineConverter::new()),
        Box::new(ClaudeCodeConverter::new()),
        Box::new(GooseConverter::new()),
    ];

    for converter in converters {
        // All should be able to convert to tool format
        let result = converter.convert_to_tool_format(&rule);
        assert!(result.is_ok());

        // All should have non-empty file extensions
        assert!(!converter.get_file_extension().is_empty());
    }
}

#[test]
fn test_all_converters_deployment_paths() {
    use std::path::Path;

    let project_root = Path::new("/tmp/test-project");

    let cursor = CursorConverter::new();
    let cursor_path = cursor.get_deployment_path(project_root);
    assert_eq!(cursor_path, project_root.join(".cursor/rules"));

    let cline = ClineConverter::new();
    let cline_path = cline.get_deployment_path(project_root);
    assert_eq!(cline_path, project_root.join(".clinerules"));

    let claude = ClaudeCodeConverter::new();
    let claude_path = claude.get_deployment_path(project_root);
    assert_eq!(claude_path, project_root.to_path_buf());

    let goose = GooseConverter::new();
    let goose_path = goose.get_deployment_path(project_root);
    assert_eq!(goose_path, project_root.to_path_buf());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulesify::models::rule::{UniversalRule, RuleMetadata, RuleContent, ContentFormat, RuleCondition, FileReference};
    use std::collections::HashMap;

    #[test]
    fn test_cursor_import_basic() {
        let converter = CursorConverter::new();
        let input = r#"---
description: Test Rule
notes: A test rule for validation
alwaysApply: true
globs:
  - "src/**/*.ts"
  - "src/**/*.tsx"
---

# Main Section

This is the main content of the rule.

## Code Standards

Follow these standards:
- Use TypeScript
- Write tests

@README.md
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "Test Rule");
        assert_eq!(result.metadata.description, Some("A test rule for validation".to_string()));
        assert!(result.metadata.auto_apply);
        assert_eq!(result.content.len(), 2);
        assert_eq!(result.content[0].title, "Main Section");
        assert_eq!(result.content[1].title, "Code Standards");
        assert_eq!(result.conditions.len(), 2);
        assert_eq!(result.references.len(), 1);
        assert_eq!(result.references[0].path, "README.md");
    }

    #[test]
    fn test_cursor_import_no_frontmatter() {
        let converter = CursorConverter::new();
        let input = r#"# Simple Rule

This is a simple rule without frontmatter.

## Guidelines

Follow these guidelines.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "Simple Rule");
        assert_eq!(result.content.len(), 2);
        assert_eq!(result.content[0].title, "Simple Rule");
        assert_eq!(result.content[1].title, "Guidelines");
    }

    #[test]
    fn test_cline_import_basic() {
        let converter = ClineConverter::new();
        let input = r#"# TypeScript Style Guide

This guide covers TypeScript coding standards.

## Naming Conventions

Use camelCase for variables and functions.

## Type Annotations

Always use explicit type annotations.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "TypeScript Style Guide");
        assert_eq!(result.metadata.description, Some("This guide covers TypeScript coding standards.".to_string()));
        assert_eq!(result.content.len(), 2);
        assert_eq!(result.content[0].title, "Naming Conventions");
        assert_eq!(result.content[1].title, "Type Annotations");
    }

    #[test]
    fn test_cline_import_no_description() {
        let converter = ClineConverter::new();
        let input = r#"# Simple Rule

## Guidelines

Follow these guidelines.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "Simple Rule");
        assert_eq!(result.metadata.description, None);
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].title, "Guidelines");
    }

    #[test]
    fn test_claude_code_import_basic() {
        let converter = ClaudeCodeConverter::new();
        let input = r#"# React Best Practices

These are best practices for React development.

## Component Structure

Keep components small and focused.

## State Management

Use hooks for state management.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "React Best Practices");
        assert_eq!(result.metadata.description, Some("These are best practices for React development.".to_string()));
        assert_eq!(result.content.len(), 2);
        assert_eq!(result.content[0].title, "Component Structure");
        assert_eq!(result.content[1].title, "State Management");
    }

    #[test]
    fn test_goose_import_basic() {
        let converter = GooseConverter::new();
        let input = r#"Python Coding Standards
======================

This document outlines Python coding standards.

Code Style
----------

Follow PEP 8 guidelines.
Use 4 spaces for indentation.

Testing
-------

Write unit tests for all functions.
Use pytest for testing.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "Python Coding Standards");
        assert_eq!(result.metadata.description, Some("This document outlines Python coding standards.".to_string()));
        assert_eq!(result.content.len(), 2);
        assert_eq!(result.content[0].title, "Code Style");
        assert_eq!(result.content[1].title, "Testing");
        assert_eq!(result.content[0].format, ContentFormat::PlainText);
        assert_eq!(result.content[1].format, ContentFormat::PlainText);
    }

    #[test]
    fn test_goose_import_no_sections() {
        let converter = GooseConverter::new();
        let input = r#"Simple Rule
===========

This is a simple rule with no sections.
Just plain text content.
"#;

        let result = converter.convert_from_tool_format(input).unwrap();
        assert_eq!(result.metadata.name, "Simple Rule");
        assert_eq!(result.metadata.description, Some("This is a simple rule with no sections.".to_string()));
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].title, "Content");
    }

    #[test]
    fn test_round_trip_conversion_cursor() {
        let converter = CursorConverter::new();

        // Create a test rule
        let original_rule = create_test_rule();

        // Convert to cursor format
        let cursor_format = converter.convert_to_tool_format(&original_rule).unwrap();

        // Convert back to URF
        let imported_rule = converter.convert_from_tool_format(&cursor_format).unwrap();

        // Verify key fields match
        assert_eq!(imported_rule.metadata.name, original_rule.metadata.name);
        assert_eq!(imported_rule.metadata.description, original_rule.metadata.description);
        assert_eq!(imported_rule.metadata.auto_apply, original_rule.metadata.auto_apply);
        assert_eq!(imported_rule.content.len(), original_rule.content.len());
        assert_eq!(imported_rule.conditions.len(), original_rule.conditions.len());
        assert_eq!(imported_rule.references.len(), original_rule.references.len());
    }

    #[test]
    fn test_round_trip_conversion_cline() {
        let converter = ClineConverter::new();

        // Create a test rule
        let original_rule = create_test_rule();

        // Convert to cline format
        let cline_format = converter.convert_to_tool_format(&original_rule).unwrap();

        // Convert back to URF
        let imported_rule = converter.convert_from_tool_format(&cline_format).unwrap();

        // Verify key fields match
        assert_eq!(imported_rule.metadata.name, original_rule.metadata.name);
        assert_eq!(imported_rule.metadata.description, original_rule.metadata.description);
        assert_eq!(imported_rule.content.len(), original_rule.content.len());
    }

    #[test]
    fn test_round_trip_conversion_claude_code() {
        let converter = ClaudeCodeConverter::new();

        // Create a test rule
        let original_rule = create_test_rule();

        // Convert to claude code format
        let claude_format = converter.convert_to_tool_format(&original_rule).unwrap();

        // Convert back to URF
        let imported_rule = converter.convert_from_tool_format(&claude_format).unwrap();

        // Verify key fields match
        assert_eq!(imported_rule.metadata.name, original_rule.metadata.name);
        assert_eq!(imported_rule.metadata.description, original_rule.metadata.description);
        assert_eq!(imported_rule.content.len(), original_rule.content.len());
    }

    #[test]
    fn test_round_trip_conversion_goose() {
        let converter = GooseConverter::new();

        // Create a test rule
        let original_rule = create_test_rule();

        // Convert to goose format
        let goose_format = converter.convert_to_tool_format(&original_rule).unwrap();

        // Convert back to URF
        let imported_rule = converter.convert_from_tool_format(&goose_format).unwrap();

        // Verify key fields match
        assert_eq!(imported_rule.metadata.name, original_rule.metadata.name);
        assert_eq!(imported_rule.metadata.description, original_rule.metadata.description);
        assert_eq!(imported_rule.content.len(), original_rule.content.len());
    }

    #[test]
    fn test_import_error_handling() {
        let converter = CursorConverter::new();

        // Test malformed YAML frontmatter
        let malformed_input = r#"---
description: Test Rule
invalid yaml: [
---

# Content
"#;

        let result = converter.convert_from_tool_format(malformed_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_empty_content() {
        let converter = ClineConverter::new();

        let result = converter.convert_from_tool_format("").unwrap();
        assert_eq!(result.metadata.name, "Imported Rule");
        assert_eq!(result.content.len(), 0);
    }

    fn create_test_rule() -> UniversalRule {
        UniversalRule {
            id: "test-rule".to_string(),
            version: "1.0.0".to_string(),
            metadata: RuleMetadata {
                name: "Test Rule".to_string(),
                description: Some("A test rule for validation".to_string()),
                tags: vec!["test".to_string(), "validation".to_string()],
                priority: 5,
                auto_apply: true,
            },
            content: vec![
                RuleContent {
                    title: "Guidelines".to_string(),
                    format: ContentFormat::Markdown,
                    value: "Follow these guidelines:\n- Be consistent\n- Write tests".to_string(),
                },
                RuleContent {
                    title: "Examples".to_string(),
                    format: ContentFormat::Markdown,
                    value: "```typescript\nconst example = 'hello';\n```".to_string(),
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
}
