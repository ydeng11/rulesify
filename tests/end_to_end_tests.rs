use rulesify::store::{RuleStore, file_store::FileStore};
use rulesify::converters::{
    RuleConverter,
    cursor::CursorConverter,
    cline::ClineConverter,
    claude_code::ClaudeCodeConverter,
    goose::GooseConverter,
};
use rulesify::templates::builtin::create_skeleton_for_rule;
use rulesify::models::rule::UniversalRule;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_complete_rule_creation_and_deployment_workflow() {
    // Setup temporary directories
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let project_temp_dir = TempDir::new().expect("Failed to create project temp dir");

    let store = FileStore::new(rules_temp_dir.path().to_path_buf());

    // Step 1: Create rule from skeleton
    let skeleton_content = create_skeleton_for_rule("integration-test-rule").unwrap();
    let rule: UniversalRule = serde_yaml::from_str(&skeleton_content)
        .expect("Failed to parse skeleton as YAML");

    // Step 2: Save rule to store
    store.save_rule(&rule).expect("Failed to save rule");

    // Step 3: Verify rule can be loaded
    let loaded_rule = store.load_rule("integration-test-rule")
        .expect("Failed to load rule")
        .expect("Rule not found");

    assert_eq!(loaded_rule.id, "integration-test-rule");
    assert_eq!(loaded_rule.metadata.name, "integration-test-rule Rule");

    // Step 4: Deploy to all tool formats
    let converters: Vec<(Box<dyn RuleConverter>, &str)> = vec![
        (Box::new(CursorConverter::new()), "cursor"),
        (Box::new(ClineConverter::new()), "cline"),
        (Box::new(ClaudeCodeConverter::new()), "claude-code"),
        (Box::new(GooseConverter::new()), "goose"),
    ];

    for (converter, tool_name) in converters {
        // Convert rule to tool format
        let tool_content = converter.convert_to_tool_format(&loaded_rule)
            .expect(&format!("Failed to convert rule for {}", tool_name));

        // Verify content is not empty and contains expected elements
        assert!(!tool_content.is_empty());
        assert!(tool_content.contains("integration-test-rule"));

        // Create deployment path
        let deployment_path = converter.get_deployment_path(project_temp_dir.path());

        // Ensure deployment directory exists
        if let Some(parent) = deployment_path.parent() {
            fs::create_dir_all(parent)
                .expect(&format!("Failed to create deployment dir for {}", tool_name));
        }

                // Write tool-specific file
        let file_path = if deployment_path.is_dir() || deployment_path.extension().is_none() {
            let path = deployment_path.join(format!("integration-test-rule.{}", converter.get_file_extension()));
            // Ensure the directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .expect(&format!("Failed to create parent dir for {} file", tool_name));
            }
            path
        } else {
            deployment_path
        };

        fs::write(&file_path, tool_content)
            .expect(&format!("Failed to write {} file", tool_name));

        // Verify file was created and contains expected content
        assert!(file_path.exists());
        let written_content = fs::read_to_string(&file_path)
            .expect(&format!("Failed to read {} file", tool_name));
        assert!(written_content.contains("integration-test-rule"));
    }

    // Step 5: Verify all expected files exist
    assert!(project_temp_dir.path().join(".cursor/rules/integration-test-rule.mdc").exists());
    assert!(project_temp_dir.path().join(".clinerules/integration-test-rule.md").exists());
    assert!(project_temp_dir.path().join("integration-test-rule.md").exists());
    assert!(project_temp_dir.path().join("integration-test-rule.goosehints").exists());
}

#[test]
fn test_multiple_rules_workflow() {
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let store = FileStore::new(rules_temp_dir.path().to_path_buf());

    // Create multiple rules
    let rule_names = vec!["rule-one", "rule-two", "rule-three"];

    for rule_name in &rule_names {
        let skeleton_content = create_skeleton_for_rule(rule_name).unwrap();
        let rule: UniversalRule = serde_yaml::from_str(&skeleton_content)
            .expect("Failed to parse skeleton as YAML");

        store.save_rule(&rule).expect("Failed to save rule");
    }

    // Verify all rules can be listed
    let listed_rules = store.list_rules().expect("Failed to list rules");
    assert_eq!(listed_rules.len(), 3);

    // Rules should be in sorted order
    assert_eq!(listed_rules[0], "rule-one");
    assert_eq!(listed_rules[1], "rule-three");
    assert_eq!(listed_rules[2], "rule-two");

    // Verify each rule can be loaded
    for rule_name in &rule_names {
        let loaded_rule = store.load_rule(rule_name)
            .expect("Failed to load rule")
            .expect("Rule not found");

        assert_eq!(loaded_rule.id, *rule_name);
    }
}

#[test]
fn test_format_specific_content_preservation() {
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let store = FileStore::new(rules_temp_dir.path().to_path_buf());

    // Create a rule with special formatting
    let skeleton_content = create_skeleton_for_rule("format-test").unwrap();
    let mut rule: UniversalRule = serde_yaml::from_str(&skeleton_content)
        .expect("Failed to parse skeleton as YAML");

    // Add specific content that should be preserved
    rule.content[0].value = "• **Bold** text\n• *Italic* text\n• `Code` snippets\n\n```rust\nfn example() {\n    println!(\"Hello!\");\n}\n```".to_string();

    store.save_rule(&rule).expect("Failed to save rule");

    // Test each converter preserves formatting
    let converters: Vec<Box<dyn RuleConverter>> = vec![
        Box::new(CursorConverter::new()),
        Box::new(ClineConverter::new()),
        Box::new(ClaudeCodeConverter::new()),
        Box::new(GooseConverter::new()),
    ];

    for converter in converters {
        let tool_content = converter.convert_to_tool_format(&rule)
            .expect("Failed to convert rule");

        // Check that markdown formatting is preserved
        assert!(tool_content.contains("**Bold**"));
        assert!(tool_content.contains("*Italic*"));
        assert!(tool_content.contains("`Code`"));
        assert!(tool_content.contains("```rust"));
        assert!(tool_content.contains("println!"));
    }
}
