use rulesify::models::rule::{ContentFormat, RuleContent, RuleMetadata, UniversalRule};
use rulesify::store::{file_store::FileStore, memory_store::MemoryStore, RuleStore};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_rule(id: &str) -> UniversalRule {
    UniversalRule {
        id: id.to_string(),
        version: "1.0.0".to_string(),
        metadata: RuleMetadata {
            name: format!("{} Rule", id),
            description: Some(format!("Test rule {}", id)),
            tags: vec!["test".to_string()],
            priority: 5,
        },
        content: vec![RuleContent {
            title: "Test Content".to_string(),
            format: ContentFormat::Markdown,
            value: "Test content value".to_string(),
        }],
        references: vec![],
        conditions: vec![],
        tool_overrides: HashMap::new(),
    }
}

#[test]
fn test_file_store_save_and_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    let rule = create_test_rule("test-rule");

    // Save rule
    let save_result = store.save_rule(&rule);
    assert!(save_result.is_ok());

    // Load rule
    let load_result = store.load_rule("test-rule");
    assert!(load_result.is_ok());

    let loaded_rule = load_result.unwrap();
    assert!(loaded_rule.is_some());

    let loaded_rule = loaded_rule.unwrap();
    assert_eq!(loaded_rule.id, "test-rule");
    assert_eq!(loaded_rule.metadata.name, "test-rule Rule");
    assert_eq!(loaded_rule.version, "1.0.0");
}

#[test]
fn test_file_store_list_rules() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    // Initially empty
    let list_result = store.list_rules();
    assert!(list_result.is_ok());
    assert!(list_result.unwrap().is_empty());

    // Add some rules
    let rule1 = create_test_rule("rule-1");
    let rule2 = create_test_rule("rule-2");

    store.save_rule(&rule1).expect("Failed to save rule1");
    store.save_rule(&rule2).expect("Failed to save rule2");

    // List should return both rules in sorted order
    let list_result = store.list_rules();
    assert!(list_result.is_ok());

    let rules = list_result.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0], "rule-1");
    assert_eq!(rules[1], "rule-2");
}

#[test]
fn test_file_store_delete_rule() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    let rule = create_test_rule("delete-me");

    // Save and verify exists
    store.save_rule(&rule).expect("Failed to save rule");
    let loaded = store.load_rule("delete-me").expect("Failed to load rule");
    assert!(loaded.is_some());

    // Delete rule
    let delete_result = store.delete_rule("delete-me");
    assert!(delete_result.is_ok());

    // Verify it's gone
    let loaded = store.load_rule("delete-me").expect("Failed to load rule");
    assert!(loaded.is_none());
}

#[test]
fn test_memory_store_creation() {
    let store = MemoryStore::new();

    // Should be empty initially
    let list_result = store.list_rules();
    assert!(list_result.is_ok());
    assert!(list_result.unwrap().is_empty());
}

#[test]
fn test_store_trait_implementations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test that both stores implement the trait
    let stores: Vec<Box<dyn RuleStore>> = vec![
        Box::new(FileStore::new(temp_dir.path().to_path_buf())),
        Box::new(MemoryStore::new()),
    ];

    for store in stores {
        // All methods should be callable through the trait
        let list_result = store.list_rules();
        assert!(list_result.is_ok());

        let load_result = store.load_rule("test");
        assert!(load_result.is_ok());

        let rule = create_test_rule("trait-test");
        let save_result = store.save_rule(&rule);
        assert!(save_result.is_ok());

        let delete_result = store.delete_rule("trait-test");
        assert!(delete_result.is_ok());
    }
}
