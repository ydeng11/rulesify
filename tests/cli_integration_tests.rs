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
    assert!(deployed_content.contains("description: |"));
    assert!(deployed_content.contains("A test rule for deployment"));
    assert!(deployed_content.contains("notes: \"Rule: Deploy Test Rule\""));
    assert!(deployed_content.contains("# Test Guidelines"));
}

#[test]
fn test_cli_deploy_merge_multiple_rules() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

    // Create test rules with different priorities
    let high_priority_rule = r#"
id: high-priority-rule
version: 1.0.0
metadata:
  name: High Priority Rule
  description: |
    This is a high priority rule for TypeScript
  tags: [typescript, style]
  priority: 8
content:
  - title: TypeScript Guidelines
    format: markdown
    value: |-
      • Use strict mode
      • Prefer const over let
references: []
conditions: []
tool_overrides:
  cursor:
    apply_mode: intelligent
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let medium_priority_rule = r#"
id: medium-priority-rule
version: 1.0.0
metadata:
  name: Medium Priority Rule
  description: |
    Testing guidelines for the project
  tags: [testing, jest, quality]
  priority: 6
content:
  - title: Testing Standards
    format: markdown
    value: |-
      • Write unit tests
      • Use descriptive test names
references: []
conditions: []
tool_overrides:
  cursor:
    apply_mode: always
  cline: {}
  claude-code: {}
  goose: {}
"#;

    let low_priority_rule = r#"
id: low-priority-rule
version: 1.0.0
metadata:
  name: Low Priority Rule
  description: |
    Documentation standards
  tags: [documentation, readme]
  priority: 3
content:
  - title: Documentation Rules
    format: markdown
    value: |-
      • Keep README up to date
      • Document public APIs
references: []
conditions: []
tool_overrides:
  cursor: {}
  cline: {}
  claude-code: {}
  goose: {}
"#;

    // Write rule files
    fs::write(
        rules_dir.join("high-priority-rule.urf.yaml"),
        high_priority_rule,
    )
    .expect("Failed to write high priority rule");
    fs::write(
        rules_dir.join("medium-priority-rule.urf.yaml"),
        medium_priority_rule,
    )
    .expect("Failed to write medium priority rule");
    fs::write(
        rules_dir.join("low-priority-rule.urf.yaml"),
        low_priority_rule,
    )
    .expect("Failed to write low priority rule");

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

    // Note: This test can't fully test the interactive merge since it requires user input
    // We would need to mock stdin or create a non-interactive mode for full testing

    // Test that the command properly parses multiple rules and validates they exist
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "deploy",
            "--tool",
            "cursor",
            "--rule",
            "high-priority-rule,medium-priority-rule,low-priority-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify deploy with multiple rules");

    // The command should fail with a prompt for merged rule ID (since we can't provide input in this test)
    // But it should get past the validation stage and show the merge preview
    assert!(
        stdout.contains("Multiple rules detected for merging")
            || stderr.contains("Multiple rules detected for merging")
    );
    assert!(stdout.contains("high-priority-rule") || stderr.contains("high-priority-rule"));
    assert!(stdout.contains("medium-priority-rule") || stderr.contains("medium-priority-rule"));
    assert!(stdout.contains("low-priority-rule") || stderr.contains("low-priority-rule"));
}

#[test]
fn test_cli_deploy_nonexistent_rule() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("Failed to create rules directory");

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

    // Test deploying a mix of existing and non-existing rules
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &[
            "--config",
            config_file.to_str().unwrap(),
            "deploy",
            "--tool",
            "cursor",
            "--rule",
            "nonexistent-rule,another-missing-rule",
        ],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify deploy with nonexistent rules");

    // Should fail early with validation error
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("not found") || stdout.contains("not found"));
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

#[test]
fn test_cli_completion_command_bash() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test bash completion generation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "bash"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion bash");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("_rulesify"), "Bash completion should contain function name");
    assert!(stdout.contains("complete"), "Bash completion should contain complete command");
    assert!(stdout.len() > 100, "Bash completion should be substantial");
}

#[test]
fn test_cli_completion_command_zsh() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test zsh completion generation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "zsh"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion zsh");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("_rulesify"), "Zsh completion should contain function name");
    assert!(stdout.contains("compdef"), "Zsh completion should contain compdef");
    assert!(stdout.len() > 100, "Zsh completion should be substantial");
}

#[test]
fn test_cli_completion_command_fish() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test fish completion generation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "fish"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion fish");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("complete"), "Fish completion should contain complete command");
    assert!(stdout.contains("rulesify"), "Fish completion should reference rulesify");
    assert!(stdout.len() > 100, "Fish completion should be substantial");
}

#[test]
fn test_cli_completion_command_powershell() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test PowerShell completion generation
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "powershell"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion powershell");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Register-ArgumentCompleter"), "PowerShell completion should contain Register-ArgumentCompleter");
    assert!(stdout.contains("rulesify"), "PowerShell completion should reference rulesify");
    assert!(stdout.len() > 100, "PowerShell completion should be substantial");
}

#[test]
fn test_cli_completion_command_invalid_shell() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test invalid shell
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "invalid-shell"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion with invalid shell");

    assert_ne!(exit_code, 0, "Command should fail with invalid shell");
    assert!(
        stdout.contains("invalid value") || stderr.contains("invalid value") ||
        stdout.contains("possible values") || stderr.contains("possible values"),
        "Should indicate invalid shell value"
    );
}

#[test]
fn test_cli_completion_command_help() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test completion help
    let (stdout, stderr, exit_code) = run_rulesify_command(
        &["completion", "--help"],
        temp_dir.path(),
    )
    .expect("Failed to run rulesify completion --help");

    assert_eq!(exit_code, 0, "Command failed with stderr: {}", stderr);
    assert!(stdout.contains("Generate shell completion scripts"), "Help should describe completion functionality");
    assert!(stdout.contains("bash"), "Help should mention bash");
    assert!(stdout.contains("zsh"), "Help should mention zsh");
    assert!(stdout.contains("fish"), "Help should mention fish");
    assert!(stdout.contains("powershell"), "Help should mention powershell");
}
