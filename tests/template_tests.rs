use rulesify::templates::{builtin::*, engine::TemplateEngine};
use std::collections::HashMap;

#[test]
fn test_get_default_skeleton() {
    let skeleton = get_default_skeleton();

    // Check basic structure
    assert!(skeleton.contains("id: <rule_id>"));
    assert!(skeleton.contains("version: 0.1.0"));
    assert!(skeleton.contains("metadata:"));
    assert!(skeleton.contains("content:"));
    assert!(skeleton.contains("tool_overrides:"));

    // Check placeholders
    assert!(skeleton.contains("<rule_id>"));
    assert!(skeleton.contains("<Human-friendly Name>"));
    assert!(skeleton.contains("<One-sentence description"));

    // Check tool override sections
    assert!(skeleton.contains("cursor:"));
    assert!(skeleton.contains("cline: {}"));
    assert!(skeleton.contains("claude-code: {}"));
    assert!(skeleton.contains("goose: {}"));

    // Check comments for guidance
    assert!(skeleton.contains("# machine-safe slug"));
    assert!(skeleton.contains("# appears in exported Markdown H1"));
}

#[test]
fn test_create_skeleton_for_rule() {
    let skeleton = create_skeleton_for_rule("my-test-rule");
    assert!(skeleton.is_ok());

    let filled = skeleton.unwrap();

    // Check that placeholders are replaced
    assert!(filled.contains("id: my-test-rule"));
    assert!(filled.contains("name: \"my-test-rule Rule\""));
    assert!(filled.contains("description: |\n    Guidelines for my-test-rule"));

    // Check that original structure is preserved
    assert!(filled.contains("version: 0.1.0"));
    assert!(filled.contains("priority: 5"));
    assert!(filled.contains("auto_apply: false"));

    // Placeholders should be gone
    assert!(!filled.contains("<rule_id>"));
    assert!(!filled.contains("<Human-friendly Name>"));
    assert!(!filled.contains("<One-sentence description"));
}

#[test]
fn test_create_skeleton_with_special_characters() {
    let skeleton = create_skeleton_for_rule("test-rule-with-dashes");
    assert!(skeleton.is_ok());

    let filled = skeleton.unwrap();
    assert!(filled.contains("id: test-rule-with-dashes"));
    assert!(filled.contains("name: \"test-rule-with-dashes Rule\""));
}

#[test]
fn test_template_engine_simple_replacement() {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, welcome to {{place}}!";

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "World".to_string());
    variables.insert("place".to_string(), "Rust".to_string());

    let result = engine.render(template, &variables);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert_eq!(rendered, "Hello World, welcome to Rust!");
}

#[test]
fn test_template_engine_missing_variable() {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, missing {{missing}}!";

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), "World".to_string());

    let result = engine.render(template, &variables);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    // Missing variables should remain as-is
    assert_eq!(rendered, "Hello World, missing {{missing}}!");
}

#[test]
fn test_template_engine_multiline() {
    let engine = TemplateEngine::new();
    let template = "Title: {{title}}\n\nContent:\n{{content}}\n\nEnd.";

    let mut variables = HashMap::new();
    variables.insert("title".to_string(), "Test Document".to_string());
    variables.insert("content".to_string(), "Line 1\nLine 2".to_string());

    let result = engine.render(template, &variables);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    let expected = "Title: Test Document\n\nContent:\nLine 1\nLine 2\n\nEnd.";
    assert_eq!(rendered, expected);
}
