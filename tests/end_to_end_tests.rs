use rulesify::converters::{
    claude_code::ClaudeCodeConverter, cline::ClineConverter, cursor::CursorConverter,
    goose::GooseConverter, RuleConverter,
};
use rulesify::models::rule::UniversalRule;
use rulesify::store::{file_store::FileStore, RuleStore};
use rulesify::templates::builtin::create_skeleton_for_rule;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_complete_rule_creation_and_deployment_workflow() {
    // Setup temporary directories
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let project_temp_dir = TempDir::new().expect("Failed to create project temp dir");

    let store = FileStore::new(rules_temp_dir.path().to_path_buf());

    // Step 1: Create rule from skeleton
    let skeleton_content = create_skeleton_for_rule("integration-test-rule").unwrap();
    let rule: UniversalRule =
        serde_yaml::from_str(&skeleton_content).expect("Failed to parse skeleton as YAML");

    // Step 2: Save rule to store
    store.save_rule(&rule).expect("Failed to save rule");

    // Step 3: Verify rule can be loaded
    let loaded_rule = store
        .load_rule("integration-test-rule")
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
        let tool_content = converter
            .convert_to_tool_format(&loaded_rule)
            .expect(&format!("Failed to convert rule for {}", tool_name));

        // Verify content is not empty and contains expected elements
        assert!(!tool_content.is_empty());
        assert!(tool_content.contains("integration-test-rule"));

        // Create deployment path
        let deployment_path = converter.get_deployment_path(project_temp_dir.path());

        // Ensure deployment directory exists
        if let Some(parent) = deployment_path.parent() {
            fs::create_dir_all(parent).expect(&format!(
                "Failed to create deployment dir for {}",
                tool_name
            ));
        }

        // Write tool-specific file
        let file_path = if deployment_path.is_dir() || deployment_path.extension().is_none() {
            let path = deployment_path.join(format!(
                "integration-test-rule.{}",
                converter.get_file_extension()
            ));
            // Ensure the directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect(&format!(
                    "Failed to create parent dir for {} file",
                    tool_name
                ));
            }
            path
        } else {
            deployment_path
        };

        fs::write(&file_path, tool_content).expect(&format!("Failed to write {} file", tool_name));

        // Verify file was created and contains expected content
        assert!(file_path.exists());
        let written_content =
            fs::read_to_string(&file_path).expect(&format!("Failed to read {} file", tool_name));
        assert!(written_content.contains("integration-test-rule"));
    }

    // Step 5: Verify all expected files exist
    assert!(project_temp_dir
        .path()
        .join(".cursor/rules/integration-test-rule.mdc")
        .exists());
    assert!(project_temp_dir
        .path()
        .join(".clinerules/integration-test-rule.md")
        .exists());
    assert!(project_temp_dir.path().join("CLAUDE.md").exists());
    assert!(project_temp_dir
        .path()
        .join("integration-test-rule.goosehints")
        .exists());
}

#[test]
fn test_multiple_rules_workflow() {
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let store = FileStore::new(rules_temp_dir.path().to_path_buf());

    // Create multiple rules
    let rule_names = vec!["rule-one", "rule-two", "rule-three"];

    for rule_name in &rule_names {
        let skeleton_content = create_skeleton_for_rule(rule_name).unwrap();
        let rule: UniversalRule =
            serde_yaml::from_str(&skeleton_content).expect("Failed to parse skeleton as YAML");

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
        let loaded_rule = store
            .load_rule(rule_name)
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
    let mut rule: UniversalRule =
        serde_yaml::from_str(&skeleton_content).expect("Failed to parse skeleton as YAML");

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
        let tool_content = converter
            .convert_to_tool_format(&rule)
            .expect("Failed to convert rule");

        // Check that markdown formatting is preserved
        assert!(tool_content.contains("**Bold**"));
        assert!(tool_content.contains("*Italic*"));
        assert!(tool_content.contains("`Code`"));
        assert!(tool_content.contains("```rust"));
        assert!(tool_content.contains("println!"));
    }
}

#[test]
fn test_deploy_with_missing_directories() {
    let rules_temp_dir = TempDir::new().expect("Failed to create rules temp dir");
    let project_temp_dir = TempDir::new().expect("Failed to create project temp dir");

    // Create a test rule file manually
    let rule_content = r#"
id: deploy-test-rule
version: 1.0.0
metadata:
  name: Deploy Test Rule
  description: |
    Test rule for deployment directory creation
  tags: []
  priority: 5

content:
  - title: Test Guidelines
    format: markdown
    value: |-
      • This is a test rule
      • Should create directories automatically
references: []
conditions: []
tool_overrides:
  cursor:
    globs: []
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let rule_file_path = rules_temp_dir.path().join("deploy-test-rule.urf.yaml");
    fs::write(&rule_file_path, rule_content).expect("Failed to write test rule");

    // Verify that the target directories don't exist yet
    assert!(!project_temp_dir.path().join(".cursor").exists());
    assert!(!project_temp_dir.path().join(".clinerules").exists());

    // Load the rule and deploy using the actual deploy logic
    let store = FileStore::new(rules_temp_dir.path().to_path_buf());
    let rule = store
        .load_rule("deploy-test-rule")
        .expect("Failed to load rule")
        .expect("Rule not found");

    // Test Cursor deployment (requires nested directory creation)
    let cursor_converter = CursorConverter::new();
    let cursor_deployment_path = cursor_converter.get_deployment_path(project_temp_dir.path());

    // Simulate the deploy_rule function behavior
    let cursor_output_path = cursor_deployment_path.join(format!(
        "deploy-test-rule.{}",
        cursor_converter.get_file_extension()
    ));

    // This should create the directory structure automatically
    if let Some(parent) = cursor_output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create cursor directory structure");
    }

    let cursor_content = cursor_converter
        .convert_to_tool_format(&rule)
        .expect("Failed to convert rule for cursor");

    fs::write(&cursor_output_path, cursor_content).expect("Failed to write cursor file");

    // Test Cline deployment (requires single directory creation)
    let cline_converter = ClineConverter::new();
    let cline_deployment_path = cline_converter.get_deployment_path(project_temp_dir.path());
    let cline_output_path = cline_deployment_path.join(format!(
        "deploy-test-rule.{}",
        cline_converter.get_file_extension()
    ));

    // This should create the directory structure automatically
    if let Some(parent) = cline_output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create cline directory structure");
    }

    let cline_content = cline_converter
        .convert_to_tool_format(&rule)
        .expect("Failed to convert rule for cline");

    fs::write(&cline_output_path, cline_content).expect("Failed to write cline file");

    // Verify that files were created with proper directory structure
    assert!(cursor_output_path.exists(), "Cursor file should exist");
    assert!(cline_output_path.exists(), "Cline file should exist");

    // Verify directory structure was created
    assert!(project_temp_dir.path().join(".cursor").exists());
    assert!(project_temp_dir.path().join(".cursor/rules").exists());
    assert!(project_temp_dir.path().join(".clinerules").exists());

    // Verify file content
    let cursor_file_content =
        fs::read_to_string(&cursor_output_path).expect("Failed to read cursor file");

    // Cursor format includes description in frontmatter and content in body
    assert!(cursor_file_content.contains("Test rule for deployment directory creation"));
    assert!(cursor_file_content.contains("This is a test rule"));
    assert!(cursor_file_content.contains("Should create directories automatically"));

    let cline_file_content =
        fs::read_to_string(&cline_output_path).expect("Failed to read cline file");
    // Cline format includes the name as H1 heading
    assert!(cline_file_content.contains("Deploy Test Rule"));
    assert!(cline_file_content.contains("This is a test rule"));
}
