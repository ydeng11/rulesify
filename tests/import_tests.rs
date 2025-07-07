use rulesify::converters::{
    claude_code::ClaudeCodeConverter, cline::ClineConverter, cursor::CursorConverter,
    goose::GooseConverter, RuleConverter,
};
use rulesify::models::rule::{
    ContentFormat, FileReference, RuleCondition, RuleContent, RuleMetadata, UniversalRule,
};
use std::collections::HashMap;

#[test]
fn test_cursor_import_with_complex_frontmatter() {
    let cursor_content = r#"---
description: Comprehensive guidelines for TypeScript development
notes: "Rule: Advanced TypeScript Rules"
globs:
  - "src/**/*.ts"
  - "src/**/*.tsx"
  - "tests/**/*.ts"
alwaysApply: true
tags:
  - typescript
  - frontend
---

# TypeScript Style Guide

Modern TypeScript development guidelines.

## Type Definitions

Use explicit types where beneficial:

```typescript
interface User {
  id: string;
  name: string;
  email: string;
}
```

## Error Handling

Prefer Result types over throwing exceptions.

@tsconfig.json
@package.json
"#;

    let converter = CursorConverter::new();
    let result = converter.convert_from_tool_format(cursor_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.metadata.name, "Advanced TypeScript Rules");
    assert_eq!(
        rule.metadata.description,
        Some("Comprehensive guidelines for TypeScript development".to_string())
    );
    // Check that auto_apply is stored in cursor tool overrides
    let cursor_overrides = rule.tool_overrides.get("cursor").unwrap();
    let auto_apply = cursor_overrides
        .get("auto_apply")
        .unwrap()
        .as_bool()
        .unwrap();
    assert_eq!(auto_apply, true);
    assert_eq!(rule.content.len(), 3);
    assert_eq!(rule.content[0].title, "TypeScript Style Guide");
    assert_eq!(rule.content[1].title, "Type Definitions");
    assert_eq!(rule.content[2].title, "Error Handling");
    assert_eq!(rule.references.len(), 2);
    assert_eq!(rule.references[0].path, "tsconfig.json");
    assert_eq!(rule.references[1].path, "package.json");
    assert_eq!(rule.conditions.len(), 3);

    if let RuleCondition::FilePattern { value } = &rule.conditions[0] {
        assert_eq!(value, "src/**/*.ts");
    } else {
        panic!("Expected FilePattern condition");
    }
}

#[test]
fn test_cline_import_with_multiple_sections() {
    let cline_content = r#"# React Best Practices

Guidelines for writing maintainable React code.

## Component Structure

Keep components small and focused.

### Functional Components

Prefer functional components over class components:

```jsx
const MyComponent = ({ props }) => {
  return <div>{props.title}</div>;
};
```

## State Management

Use hooks for state management:

- useState for local state
- useContext for shared state
- useReducer for complex state logic

## Testing

Write tests for all components using Jest and React Testing Library.
"#;

    let converter = ClineConverter::new();
    let result = converter.convert_from_tool_format(cline_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.metadata.name, "React Best Practices");
    assert_eq!(
        rule.metadata.description,
        Some("Guidelines for writing maintainable React code.".to_string())
    );
    assert_eq!(rule.content.len(), 3);
    assert_eq!(rule.content[0].title, "Component Structure");
    assert_eq!(rule.content[1].title, "State Management");
    assert_eq!(rule.content[2].title, "Testing");
    assert!(rule.content[0].value.contains("Functional Components"));
    assert!(rule.content[1].value.contains("useState"));
}

#[test]
fn test_claude_code_import_minimal() {
    let claude_content = r#"# Python Code Style

Use black for formatting and type hints everywhere.
"#;

    let converter = ClaudeCodeConverter::new();
    let result = converter.convert_from_tool_format(claude_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.metadata.name, "Python Code Style");
    assert_eq!(
        rule.metadata.description,
        Some("Use black for formatting and type hints everywhere.".to_string())
    );
    // The minimal content becomes a "Content" section
    assert_eq!(rule.content.len(), 1);
    if !rule.content.is_empty() {
        assert_eq!(rule.content[0].title, "Content");
    }
}

#[test]
fn test_goose_import_with_underlined_sections() {
    let goose_content = r#"Database Design Guidelines
==========================

Best practices for designing robust database schemas.

Schema Design
-------------

- Use meaningful table and column names
- Implement proper foreign key constraints
- Consider indexing strategy early

Performance Optimization
------------------------

- Use EXPLAIN ANALYZE for query optimization
- Implement connection pooling
- Monitor query performance regularly

Data Migration
--------------

- Always backup before migrations
- Test migrations on staging first
- Use transaction-safe migration scripts
"#;

    let converter = GooseConverter::new();
    let result = converter.convert_from_tool_format(goose_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.metadata.name, "Database Design Guidelines");
    assert_eq!(
        rule.metadata.description,
        Some("Best practices for designing robust database schemas.".to_string())
    );
    assert_eq!(rule.content.len(), 3);
    assert_eq!(rule.content[0].title, "Schema Design");
    assert_eq!(rule.content[1].title, "Performance Optimization");
    assert_eq!(rule.content[2].title, "Data Migration");
    assert!(rule.content[0].value.contains("meaningful table"));
    assert!(rule.content[1].value.contains("EXPLAIN ANALYZE"));
    assert!(rule.content[2].value.contains("backup before migrations"));
}

#[test]
fn test_import_malformed_content() {
    let malformed_cursor = r#"---
description: Test
invalid_yaml: [unclosed array
---
# Test
"#;

    let converter = CursorConverter::new();
    let result = converter.convert_from_tool_format(malformed_cursor);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse YAML frontmatter"));
}

#[test]
fn test_import_empty_files() {
    let converters: Vec<Box<dyn RuleConverter>> = vec![
        Box::new(CursorConverter::new()),
        Box::new(ClineConverter::new()),
        Box::new(ClaudeCodeConverter::new()),
        Box::new(GooseConverter::new()),
    ];

    for converter in converters {
        let result = converter.convert_from_tool_format("");
        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.metadata.name, "Imported Rule");
        assert!(rule.content.is_empty() || rule.content.len() == 1);
    }
}

#[test]
fn test_import_only_whitespace() {
    let whitespace_content = "   \n\n\t  \n   ";

    let converter = ClineConverter::new();
    let result = converter.convert_from_tool_format(whitespace_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.metadata.name, "Imported Rule");
    assert!(rule.content.is_empty());
}

#[test]
fn test_import_preserves_code_blocks() {
    let cline_content = r#"# Code Examples

## JavaScript

```javascript
function example() {
  return "Hello, World!";
}
```

## Python

```python
def example():
    return "Hello, World!"
```
"#;

    let converter = ClineConverter::new();
    let result = converter.convert_from_tool_format(cline_content);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert_eq!(rule.content.len(), 2);
    assert!(rule.content[0].value.contains("```javascript"));
    assert!(rule.content[0].value.contains("function example()"));
    assert!(rule.content[1].value.contains("```python"));
    assert!(rule.content[1].value.contains("def example():"));
}

#[test]
fn test_import_special_characters() {
    let content_with_special_chars = r#"# Special Characters Test

Content with special characters: áéíóú, ñ, ü, ß, 中文, 日本語, 🚀, 💻

## Code with Unicode

```javascript
const emoji = "🎉";
const chinese = "你好";
```
"#;

    let converter = ClineConverter::new();
    let result = converter.convert_from_tool_format(content_with_special_chars);
    assert!(result.is_ok());

    let rule = result.unwrap();
    assert!(rule.metadata.name.contains("Special Characters"));

    // The content should have the code section with Unicode
    assert_eq!(rule.content.len(), 1);
    assert_eq!(rule.content[0].title, "Code with Unicode");

    // Check for the JavaScript code with emojis and Chinese characters
    let code_content = &rule.content[0].value;
    assert!(code_content.contains("🎉"), "Code emoji not found");
    assert!(
        code_content.contains("你好"),
        "Chinese characters not found"
    );
    assert!(
        code_content.contains("```javascript"),
        "JavaScript code block not found"
    );
}

#[test]
fn test_import_rule_id_generation() {
    let test_cases = vec![
        ("Simple Rule", "simple-rule"),
        ("Rule With Spaces", "rule-with-spaces"),
        ("Rule_With_Underscores", "rule-with-underscores"),
        ("Rule123", "rule123"),
        ("Rule-With-Hyphens", "rule-with-hyphens"),
        (
            "Rule With Special!@#$%^&*()Characters",
            "rule-with-specialcharacters",
        ),
    ];

    for (rule_name, expected_id) in test_cases {
        let content = format!("# {}\n\nTest content", rule_name);
        let converter = ClineConverter::new();
        let result = converter.convert_from_tool_format(&content);
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.id, expected_id);
    }
}

#[test]
fn test_import_round_trip_all_tools() {
    let original_rule = UniversalRule {
        id: "test-rule".to_string(),
        version: "1.0.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("A test rule for validation".to_string()),
            tags: vec!["test".to_string(), "validation".to_string()],
            priority: 7,
        },
        content: vec![
            RuleContent {
                title: "Guidelines".to_string(),
                format: ContentFormat::Markdown,
                value: "Follow these guidelines:\n\n- Be consistent\n- Write tests".to_string(),
            },
            RuleContent {
                title: "Examples".to_string(),
                format: ContentFormat::Markdown,
                value: "```javascript\nconst example = true;\n```".to_string(),
            },
        ],
        references: vec![FileReference {
            path: "README.md".to_string(),
        }],
        conditions: vec![RuleCondition::FilePattern {
            value: "src/**/*.js".to_string(),
        }],
        tool_overrides: HashMap::new(),
    };

    let tools: Vec<Box<dyn RuleConverter>> = vec![
        Box::new(CursorConverter::new()),
        Box::new(ClineConverter::new()),
        Box::new(ClaudeCodeConverter::new()),
        Box::new(GooseConverter::new()),
    ];

    for converter in tools {
        // Export to tool format
        let tool_format = converter.convert_to_tool_format(&original_rule);
        assert!(tool_format.is_ok());

        // Import back from tool format
        let imported_rule = converter.convert_from_tool_format(&tool_format.unwrap());
        assert!(imported_rule.is_ok());

        let imported = imported_rule.unwrap();

        // Verify key properties are preserved
        assert_eq!(imported.metadata.name, original_rule.metadata.name);
        assert!(!imported.content.is_empty());
        assert_eq!(imported.content[0].title, "Guidelines");

        // Note: Some properties may differ due to tool format limitations
        // but the essential content should be preserved
    }
}
