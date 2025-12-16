use rulesify::cli::commands::sync;
use rulesify::models::rule::{ContentFormat, RuleContent, RuleMetadata, UniversalRule};
use rulesify::store::{file_store::FileStore, RuleStore};
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to serialize tests that change the current working directory
// This prevents race conditions when tests run in parallel
static DIR_CHANGE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_sync_preserves_original_rule_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a rules directory
    let rules_dir = temp_path.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();

    // Create a .cursor/rules directory
    let cursor_dir = temp_path.join(".cursor/rules");
    fs::create_dir_all(&cursor_dir).unwrap();

    // Create an original URF rule with ID "rust-programming"
    let original_rule = UniversalRule {
        id: "rust-programming".to_string(),
        version: "0.1.0".to_string(),
        metadata: RuleMetadata {
            name: "The Golden Rule of Rust Programming".to_string(),
            description: Some("Embrace the Borrow Checker".to_string()),
            tags: vec!["rust".to_string()],
            priority: 10,
        },
        content: vec![RuleContent {
            title: "Core Principles".to_string(),
            format: ContentFormat::Markdown,
            value: "Follow Rust best practices".to_string(),
        }],
        references: vec![],
        conditions: vec![],
        tool_overrides: std::collections::HashMap::new(),
    };

    // Save the original rule
    let store = FileStore::new(rules_dir.clone());
    store.save_rule(&original_rule).unwrap();

    // Create a modified cursor file with the same filename but different content
    let cursor_file = cursor_dir.join("rust-programming.mdc");
    let cursor_content = r#"---
description: The Golden Rule of Rust Programming
notes: |
  Embrace the Borrow Checker
  Modified content for sync test
alwaysApply: true
---

# Core Principles

Follow Rust best practices - MODIFIED CONTENT

# New Section

This is a new section added to test sync functionality.
"#;

    fs::write(&cursor_file, cursor_content).unwrap();

    // Create a minimal config
    let config_dir = temp_path.join(".rulesify");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.yaml");
    let config_content = format!(
        r#"
rules_directory: {}
default_tools:
  - cursor
"#,
        rules_dir.display()
    );
    fs::write(&config_file, config_content).unwrap();

    // Run sync command (changing directory to temp_path for the operation)
    // Use a lock to prevent race conditions with other tests
    let _lock = DIR_CHANGE_LOCK.lock().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    let result = sync::run(
        false,                                // not dry run
        Some("rust-programming".to_string()), // specific rule
        Some("cursor".to_string()),           // specific tool
        Some(config_file),
    );

    // Restore original directory before any assertions
    let _ = std::env::set_current_dir(&original_dir);
    drop(_lock); // Explicitly drop the lock before assertions

    assert!(result.is_ok(), "Sync should succeed: {:?}", result);

    // Verify the rule was updated, not created with a new ID
    let updated_rule = store.load_rule("rust-programming").unwrap();
    assert!(updated_rule.is_some(), "Original rule should still exist");

    let updated_rule = updated_rule.unwrap();
    assert_eq!(
        updated_rule.id, "rust-programming",
        "Rule ID should be preserved"
    );
    assert_eq!(
        updated_rule.metadata.name,
        "The Golden Rule of Rust Programming"
    );

    // Verify content was updated
    assert_eq!(
        updated_rule.content.len(),
        2,
        "Should have 2 content sections"
    );
    assert_eq!(updated_rule.content[0].title, "Core Principles");
    assert!(updated_rule.content[0].value.contains("MODIFIED CONTENT"));
    assert_eq!(updated_rule.content[1].title, "New Section");

    // Verify no new rule was created with the generated ID
    // Check that the generated ID rule file doesn't exist
    let generated_id_path = rules_dir.join("the-golden-rule-of-rust-programming.urf.yaml");
    assert!(
        !generated_id_path.exists(),
        "No rule file should be created with generated ID"
    );
}

#[test]
fn test_sync_warns_when_no_existing_urf() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a rules directory (empty)
    let rules_dir = temp_path.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();

    // Create a .cursor/rules directory
    let cursor_dir = temp_path.join(".cursor/rules");
    fs::create_dir_all(&cursor_dir).unwrap();

    // Create a cursor file without a corresponding URF
    let cursor_file = cursor_dir.join("new-rule.mdc");
    let cursor_content = r#"---
description: New Rule
notes: This is a new rule
alwaysApply: false
---

# Content

This is new content.
"#;

    fs::write(&cursor_file, cursor_content).unwrap();

    // Create a minimal config
    let config_dir = temp_path.join(".rulesify");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.yaml");
    let config_content = format!(
        r#"
rules_directory: {}
default_tools:
  - cursor
"#,
        rules_dir.display()
    );
    fs::write(&config_file, config_content).unwrap();

    // Run sync command (changing directory to temp_path for the operation)
    // Use a lock to prevent race conditions with other tests
    let _lock = DIR_CHANGE_LOCK.lock().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    let result = sync::run(
        false,                        // not dry run
        Some("new-rule".to_string()), // specific rule
        Some("cursor".to_string()),   // specific tool
        Some(config_file),
    );

    // Restore original directory before any assertions
    let _ = std::env::set_current_dir(&original_dir);
    drop(_lock); // Explicitly drop the lock before assertions

    assert!(result.is_ok(), "Sync should succeed: {:?}", result);

    // Verify the rule was created with the filename-based ID
    let store = FileStore::new(rules_dir);
    let created_rule = store.load_rule("new-rule").unwrap();
    assert!(created_rule.is_some(), "Rule should be created");

    let created_rule = created_rule.unwrap();
    assert_eq!(created_rule.id, "new-rule", "Rule ID should match filename");
    assert_eq!(created_rule.metadata.name, "New Rule");
}

#[test]
fn test_sync_dry_run_mode() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a rules directory
    let rules_dir = temp_path.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();

    // Create a .cursor/rules directory
    let cursor_dir = temp_path.join(".cursor/rules");
    fs::create_dir_all(&cursor_dir).unwrap();

    // Create an original URF rule
    let original_rule = UniversalRule {
        id: "test-rule".to_string(),
        version: "0.1.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("Original description".to_string()),
            tags: vec![],
            priority: 5,
        },
        content: vec![RuleContent {
            title: "Content".to_string(),
            format: ContentFormat::Markdown,
            value: "Original content".to_string(),
        }],
        references: vec![],
        conditions: vec![],
        tool_overrides: std::collections::HashMap::new(),
    };

    // Save the original rule
    let store = FileStore::new(rules_dir.clone());
    store.save_rule(&original_rule).unwrap();

    // Create a modified cursor file
    let cursor_file = cursor_dir.join("test-rule.mdc");
    let cursor_content = r#"---
description: Test Rule
notes: Modified description
alwaysApply: false
---

# Content

Modified content
"#;

    fs::write(&cursor_file, cursor_content).unwrap();

    // Create a minimal config
    let config_dir = temp_path.join(".rulesify");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.yaml");
    let config_content = format!(
        r#"
rules_directory: {}
default_tools:
  - cursor
"#,
        rules_dir.display()
    );
    fs::write(&config_file, config_content).unwrap();

    // Run sync command in dry run mode (changing directory to temp_path for the operation)
    // Use a lock to prevent race conditions with other tests
    let _lock = DIR_CHANGE_LOCK.lock().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    let result = sync::run(
        true,                          // dry run
        Some("test-rule".to_string()), // specific rule
        Some("cursor".to_string()),    // specific tool
        Some(config_file),
    );

    // Restore original directory before any assertions
    let _ = std::env::set_current_dir(&original_dir);
    drop(_lock); // Explicitly drop the lock before assertions

    assert!(result.is_ok(), "Sync should succeed: {:?}", result);

    // Verify the rule was NOT modified (dry run)
    let unchanged_rule = store.load_rule("test-rule").unwrap().unwrap();
    assert_eq!(
        unchanged_rule.metadata.description,
        Some("Original description".to_string())
    );
    assert_eq!(unchanged_rule.content[0].value, "Original content");
}
