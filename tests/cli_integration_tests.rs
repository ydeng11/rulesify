use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn get_rulesify_binary() -> String {
    // Get the current working directory and construct absolute paths
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let release_path = current_dir.join("target/release/rulesify");
    let debug_path = current_dir.join("target/debug/rulesify");

    if release_path.exists() {
        release_path.to_string_lossy().to_string()
    } else if debug_path.exists() {
        debug_path.to_string_lossy().to_string()
    } else {
        panic!(
            "Rulesify binary not found at {} or {}. Please run 'cargo build' first.",
            debug_path.display(),
            release_path.display()
        );
    }
}

fn run_rulesify_command(
    args: &[&str],
    working_dir: &Path,
) -> Result<(String, String, i32), std::io::Error> {
    let binary = get_rulesify_binary();
    let output = Command::new(&binary)
        .args(args)
        .current_dir(working_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}

#[test]
fn test_cli_help_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (stdout, _stderr, exit_code) =
        run_rulesify_command(&["--help"], temp_dir.path()).expect("Failed to run rulesify --help");

    assert_eq!(exit_code, 0);
    assert!(stdout.contains("Rulesify manages Universal Rule Files"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("rule"));
    assert!(stdout.contains("deploy"));
    assert!(stdout.contains("import"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("sync"));
    assert!(stdout.contains("config"));
}

#[test]
fn test_cli_rule_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Set up a custom config to use our temp directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_content = format!(
        r#"
rules_directory: "{}"
editor: null
default_tools:
  - cursor
  - cline
"#,
        rules_dir.display()
    );

    let config_file = config_dir.join("config.yaml");
    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test rule creation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "rule",
            "new",
            "test-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify rule new");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Created new rule: test-rule"));

    // Verify the rule file was created
    let rule_file = rules_dir.join("test-rule.urf.yaml");
    assert!(rule_file.exists(), "Rule file was not created");

    let rule_content = fs::read_to_string(&rule_file).expect("Failed to read rule file");
    assert!(rule_content.contains("id: test-rule"));
    assert!(rule_content.contains("version: 0.1.0"));

    // Test rule listing
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["--config", config_file.to_str().unwrap(), "rule", "list"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify rule list");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("test-rule"));
    assert!(stdout.contains("Rules (1)"));

    // Test rule show
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "rule",
            "show",
            "test-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify rule show");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Rule: test-rule Rule"));
    assert!(stdout.contains("ID: test-rule"));
    assert!(stdout.contains("Version: 0.1.0"));

    // Test rule deletion
    let (stdout, stderr, _exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "rule",
            "delete",
            "test-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify rule delete");

    // The delete command requires confirmation, so we expect it to prompt
    assert!(stdout.contains("Are you sure") || stderr.contains("Are you sure"));
}

#[test]
fn test_cli_validation_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Create a test rule manually
    let rule_content = r#"
id: validation-test
version: 1.0.0
metadata:
  name: Validation Test Rule
  description: |
    A test rule for validation
  tags:
    - test
    - validation
  priority: 5
content:
  - title: Test Guidelines
    format: markdown
    value: |-
      • This is a test rule
      • It should pass validation
references: []
conditions: []
tool_overrides:
  cursor: {}
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let rule_file = rules_dir.join("validation-test.urf.yaml");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Set up config
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_content = format!(
        r#"
rules_directory: "{}"
editor: null
default_tools:
  - cursor
"#,
        rules_dir.display()
    );

    let config_file = config_dir.join("config.yaml");
    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test validation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "validate",
            "--all",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify validate");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Validating 1 rule"));
    assert!(stdout.contains("validation-test"));
    assert!(stdout.contains("All rules passed validation") || stdout.contains("0 error(s)"));
}

#[test]
fn test_cli_deploy_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Create a test rule
    let rule_content = r#"
id: deploy-test
version: 1.0.0
metadata:
  name: Deploy Test Rule
  description: |
    A test rule for deployment
  tags: []
  priority: 5
content:
  - title: Test Guidelines
    format: markdown
    value: |-
      • This is a test rule
      • It should deploy successfully
references: []
conditions: []
tool_overrides:
  cursor: {}
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let rule_file = rules_dir.join("deploy-test.urf.yaml");
    fs::write(&rule_file, rule_content).expect("Failed to write rule file");

    // Set up config
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_content = format!(
        r#"
rules_directory: "{}"
editor: null
default_tools:
  - cursor
"#,
        rules_dir.display()
    );

    let config_file = config_dir.join("config.yaml");
    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test deployment
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "deploy",
            "--tool",
            "cursor",
            "--all",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify deploy");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Deploying 1 rule"));
    assert!(stdout.contains("deploy-test"));
    assert!(stdout.contains("Deployment complete"));

    // Verify the deployed file was created
    let deployed_file = temp_dir.path().join(".cursor/rules/deploy-test.mdc");
    assert!(deployed_file.exists(), "Deployed file was not created");

    let deployed_content =
        fs::read_to_string(&deployed_file).expect("Failed to read deployed file");
    assert!(deployed_content.contains("---"));
    assert!(deployed_content.contains("description: Deploy Test Rule"));
    assert!(deployed_content.contains("# Test Guidelines"));
}

#[test]
fn test_cli_import_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Create a sample Cursor rule file to import
    let cursor_content = r#"---
description: Imported Rule
notes: This rule was imported from Cursor format
globs:
  - "*.ts"
alwaysApply: false
---

# Coding Standards

Follow these coding standards:

## TypeScript

- Use explicit types
- Prefer interfaces over types

## Testing

- Write unit tests for all functions
- Use descriptive test names
"#;

    let import_file = temp_dir.path().join("sample-rule.mdc");
    fs::write(&import_file, cursor_content).expect("Failed to write import file");

    // Set up config
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_content = format!(
        r#"
rules_directory: "{}"
editor: null
default_tools:
  - cursor
"#,
        rules_dir.display()
    );

    let config_file = config_dir.join("config.yaml");
    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test import
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "import",
            "--tool",
            "cursor",
            import_file.to_str().unwrap(),
            "--rule-id",
            "imported-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify import");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Successfully imported rule: imported-rule"));
    assert!(stdout.contains("Name: Imported Rule"));

    // Verify the URF file was created
    let urf_file = rules_dir.join("imported-rule.urf.yaml");
    assert!(urf_file.exists(), "URF file was not created");

    let urf_content = fs::read_to_string(&urf_file).expect("Failed to read URF file");
    assert!(urf_content.contains("id: imported-rule"));
    assert!(urf_content.contains("name: Imported Rule"));
    assert!(urf_content.contains("title: Coding Standards"));
}

#[test]
fn test_cli_config_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Set up config
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_content = format!(
        r#"
rules_directory: "{}"
editor: vim
default_tools:
  - cursor
  - cline
"#,
        rules_dir.display()
    );

    let config_file = config_dir.join("config.yaml");
    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Test config show
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["--config", config_file.to_str().unwrap(), "config", "show"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify config show");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Rulesify Configuration"));
    assert!(stdout.contains("Rules Directory"));
    assert!(stdout.contains("Editor: vim"));
    assert!(stdout.contains("Default Tools: cursor, cline"));
}

#[test]
fn test_cli_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test with non-existent rule
    let (stdout, stderr, exit_code) =
        run_rulesify_command(&["rule", "show", "non-existent-rule"], temp_dir.path())
            .expect("Failed to run rulesify rule show");

    assert_ne!(exit_code, 0);
    assert!(stdout.contains("not found") || stderr.contains("not found"));

    // Test with invalid tool
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["deploy", "--tool", "invalid-tool", "--all"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify deploy");

    assert_ne!(exit_code, 0);
    assert!(stdout.contains("Unsupported tool") || stderr.contains("Unsupported tool"));
}

#[test]
fn test_cli_verbose_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test verbose flag
    let (stdout, _stderr, exit_code) =
        run_rulesify_command(&["--verbose", "--help"], temp_dir.path())
            .expect("Failed to run rulesify --verbose --help");

    assert_eq!(exit_code, 0);
    assert!(stdout.contains("Rulesify manages Universal Rule Files"));
}
