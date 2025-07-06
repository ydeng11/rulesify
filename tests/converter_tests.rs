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
    assert!(output.contains("description: A test rule for unit testing"));
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
