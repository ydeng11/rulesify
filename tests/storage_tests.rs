use rulesify::models::rule::{ContentFormat, RuleContent, RuleMetadata, UniversalRule};
use rulesify::store::{file_store::FileStore, memory_store::MemoryStore, RuleStore};
use std::collections::HashMap;
use std::fs;
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

#[test]
fn test_parse_rule_with_string_references() {
    // Test that we can parse a rule file with references as simple strings
    // (the format that was causing the original parsing issue)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    // Create a rule file with string references (the problematic format)
    let rule_content = r#"id: playwright-quarkus
version: 1.0.0
metadata:
  name: "Quarkus Playwright & Quinoa E2E Testing Guide"
  description: |
    Detailed guide for implementing reliable End-to-End (E2E) tests in Quarkus applications using Playwright for browser automation and Quinoa for frontend management.
  tags: [java, quarkus, playwright, quinoa, testing, e2e]
  priority: 8
content:
  - title: "Guidelines"
    format: markdown
    value: |-
      ### 1. Dependency and Configuration
      * **Dependencies**: Ensure `io.quarkiverse.quinoa:quarkus-quinoa` and `io.quarkiverse.playwright:quarkus-playwright` are included in `pom.xml`.
  - title: "Code Example"
    format: code
    value: |-
      package org.acme.e2e;
      import com.microsoft.playwright.*;
references:
  - https://docs.quarkiverse.io/quarkus-playwright/dev/index.html
  - https://docs.quarkiverse.io/quarkus-quinoa/dev/index.html
conditions:
  - type: file_pattern
    value: "pom.xml"
  - type: file_pattern
    value: "src/test/java/**/*.java"
tool_overrides:
  cursor:
    apply_mode: intelligent
    globs: ["pom.xml", "src/test/java/**/*.java"]
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let rule_file = temp_dir.path().join("playwright-quarkus.urf.yaml");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Try to load the rule - this should succeed with our fix
    let load_result = store.load_rule("playwright-quarkus");
    assert!(
        load_result.is_ok(),
        "Failed to load rule with string references"
    );

    let loaded_rule = load_result.unwrap();
    assert!(loaded_rule.is_some(), "Rule should be loaded");

    let rule = loaded_rule.unwrap();
    assert_eq!(rule.id, "playwright-quarkus");
    assert_eq!(
        rule.metadata.name,
        "Quarkus Playwright & Quinoa E2E Testing Guide"
    );
    assert_eq!(rule.references.len(), 2);
    assert_eq!(
        rule.references[0].path,
        "https://docs.quarkiverse.io/quarkus-playwright/dev/index.html"
    );
    assert_eq!(
        rule.references[1].path,
        "https://docs.quarkiverse.io/quarkus-quinoa/dev/index.html"
    );
    assert_eq!(rule.content.len(), 2);
    assert_eq!(rule.content[0].title, "Guidelines");
    assert_eq!(rule.content[1].title, "Code Example");
}

#[test]
fn test_parse_rule_with_object_references() {
    // Test that we can still parse a rule file with references as objects
    // (the alternative format that should also work)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    // Create a rule file with object references
    let rule_content = r#"id: test-rule-object-refs
version: 1.0.0
metadata:
  name: "Test Rule with Object References"
  description: "A test rule"
  tags: [test]
  priority: 5
content:
  - title: "Content"
    format: markdown
    value: "Test content"
references:
  - path: https://example.com/doc1
  - path: https://example.com/doc2
conditions: []
tool_overrides: {}
"#;

    let rule_file = temp_dir.path().join("test-rule-object-refs.urf.yaml");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Try to load the rule - this should also succeed
    let load_result = store.load_rule("test-rule-object-refs");
    assert!(
        load_result.is_ok(),
        "Failed to load rule with object references"
    );

    let loaded_rule = load_result.unwrap();
    assert!(loaded_rule.is_some(), "Rule should be loaded");

    let rule = loaded_rule.unwrap();
    assert_eq!(rule.id, "test-rule-object-refs");
    assert_eq!(rule.references.len(), 2);
    assert_eq!(rule.references[0].path, "https://example.com/doc1");
    assert_eq!(rule.references[1].path, "https://example.com/doc2");
}

#[test]
fn test_parse_playwright_quarkus_fixture() {
    // Test parsing the actual playwright-quarkus.urf.yaml file
    // This file uses string references (the format that was causing the original issue)
    let fixture_path = std::path::Path::new("tests/fixtures/playwright-quarkus.urf.yaml");

    if !fixture_path.exists() {
        // Skip test if fixture doesn't exist (e.g., in CI)
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let store = FileStore::new(temp_dir.path().to_path_buf());

    // Copy the fixture file to the temp directory
    let rule_file = temp_dir.path().join("playwright-quarkus.urf.yaml");
    fs::copy(fixture_path, &rule_file).expect("Failed to copy fixture file");

    // Try to load the rule - this should succeed with our fix
    let load_result = store.load_rule("playwright-quarkus");
    assert!(
        load_result.is_ok(),
        "Failed to load playwright-quarkus rule"
    );

    let loaded_rule = load_result.unwrap();
    assert!(loaded_rule.is_some(), "Rule should be loaded");

    let rule = loaded_rule.unwrap();
    assert_eq!(rule.id, "playwright-quarkus");
    assert_eq!(rule.version, "1.0.0");
    assert_eq!(
        rule.metadata.name,
        "Quarkus Playwright & Quinoa E2E Testing Guide"
    );
    assert_eq!(rule.metadata.priority, 8);
    assert!(rule.metadata.tags.contains(&"java".to_string()));
    assert!(rule.metadata.tags.contains(&"quarkus".to_string()));
    assert!(rule.metadata.tags.contains(&"playwright".to_string()));

    // Verify references were parsed correctly (as strings)
    assert_eq!(rule.references.len(), 2);
    assert_eq!(
        rule.references[0].path,
        "https://docs.quarkiverse.io/quarkus-playwright/dev/index.html"
    );
    assert_eq!(
        rule.references[1].path,
        "https://docs.quarkiverse.io/quarkus-quinoa/dev/index.html"
    );

    // Verify content sections
    assert_eq!(rule.content.len(), 2);
    assert_eq!(rule.content[0].title, "Guidelines");
    assert_eq!(rule.content[1].title, "Code Example");
    assert_eq!(rule.content[1].format, ContentFormat::Code);

    // Verify conditions
    assert_eq!(rule.conditions.len(), 2);
}
