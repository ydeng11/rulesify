use rulesify::models::rule::{UniversalRule, RuleMetadata, RuleContent, ContentFormat};
use std::collections::HashMap;

#[test]
fn test_universal_rule_creation() {
    let rule = UniversalRule {
        id: "test-rule".to_string(),
        version: "0.1.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("A test rule".to_string()),
            tags: vec!["test".to_string()],
            priority: 5,
            auto_apply: false,
        },
        content: vec![RuleContent {
            title: "Test Section".to_string(),
            format: ContentFormat::Markdown,
            value: "This is a test rule content".to_string(),
        }],
        references: vec![],
        conditions: vec![],
        tool_overrides: HashMap::new(),
    };

    assert_eq!(rule.id, "test-rule");
    assert_eq!(rule.metadata.name, "Test Rule");
    assert_eq!(rule.content.len(), 1);
} 