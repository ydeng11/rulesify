# Testing Patterns

**Analysis Date:** 2026-04-02

## Framework & Setup

**Runner:**
- Rust built-in test framework via `cargo test`
- No external test framework (uses `#[test]` attribute)

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!` macros
- Pattern matching for error checking: `assert!(result.is_ok())`, `assert!(result.is_err())`

**Dependencies:**
- `tempfile = "3.8"` in `[dev-dependencies]` for temporary directories
- Tests use production dependencies: `anyhow`, `serde_json`, `serde_yaml`

**Run Commands:**
```bash
cargo test                  # Run all tests
cargo test --test converter_tests    # Run specific test file
cargo test -- --nocapture   # Show stdout/stderr output
cargo test -- --show-output # Show output for passing tests
```

## Test File Organization

**Location:**
- Tests in dedicated `tests/` directory (integration tests)
- Unit tests embedded in source files with `#[cfg(test)]` module

**Naming:**
- Test files: `{module}_tests.rs` pattern
- Example files: `converter_tests.rs`, `storage_tests.rs`, `validation_tests.rs`

**Structure:**
```
tests/
├── converter_tests.rs      # Converter functionality tests
├── storage_tests.rs        # File/Memory store tests
├── validation_tests.rs     # Validator tests
├── template_tests.rs       # Template engine tests
├── sync_tests.rs           # Sync functionality tests
├── import_tests.rs         # Import command tests
├── cli_integration_tests.rs # CLI command tests
├── end_to_end_tests.rs     # Full workflow tests
└── fixtures/
    └── playwright-quarkus.urf.yaml  # Sample rule file
```

## Test Structure

**Suite Organization:**
```rust
// Top-level test functions
#[test]
fn test_cursor_converter_basic_conversion() {
    let converter = CursorConverter::new();
    let rule = create_test_rule();
    
    let result = converter.convert_to_tool_format(&rule);
    assert!(result.is_ok());
    // ...
}

// Nested test modules for grouped tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cursor_import_basic() { ... }
    
    #[test]
    fn test_round_trip_conversion_cursor() { ... }
}
```

**Embedded Unit Tests:**
- Unit tests placed in source files under `#[cfg(test)]` module
- Example from `src/cli/commands/deploy.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_rule(id: &str, name: &str, ...) -> UniversalRule { ... }
    
    #[test]
    fn test_merge_rules_priority_ordering() { ... }
    
    #[test]
    fn test_merge_rules_tag_deduplication() { ... }
}
```

## Test Helper Patterns

**Factory Functions:**
- Create test fixtures with helper functions
- Pattern: `create_test_rule()` or `create_valid_rule()`
- Example from `tests/converter_tests.rs`:
```rust
fn create_test_rule() -> UniversalRule {
    UniversalRule {
        id: "test-rule".to_string(),
        version: "1.0.0".to_string(),
        metadata: RuleMetadata {
            name: "Test Rule".to_string(),
            description: Some("A test rule for unit testing".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            priority: 5,
        },
        content: vec![...],
        references: vec![...],
        conditions: vec![...],
        tool_overrides: HashMap::new(),
    }
}
```

**Fixture Files:**
- Test fixtures in `tests/fixtures/` directory
- Example: `tests/fixtures/playwright-quarkus.urf.yaml`
- Skip tests if fixtures don't exist:
```rust
if !fixture_path.exists() {
    return; // Skip test if fixture doesn't exist (e.g., in CI)
}
```

## Mocking & Test Doubles

**In-Memory Implementations:**
- `MemoryStore` in `src/store/memory_store.rs` provides test-friendly implementation
- Used for testing without file system dependencies
- Example usage:
```rust
let store = MemoryStore::new();
// No temp directory needed
```

**Trait-Based Testing:**
- Test multiple implementations through trait interface
- Example from `tests/storage_tests.rs`:
```rust
let stores: Vec<Box<dyn RuleStore>> = vec![
    Box::new(FileStore::new(temp_dir.path().to_path_buf())),
    Box::new(MemoryStore::new()),
];

for store in stores {
    // Test trait methods work for all implementations
    let list_result = store.list_rules();
    assert!(list_result.is_ok());
}
```

**Tempfile Pattern:**
- Use `TempDir` for file system tests
- Auto-cleaned after test completion
- Pattern from `tests/storage_tests.rs`:
```rust
let temp_dir = TempDir::new().expect("Failed to create temp dir");
let store = FileStore::new(temp_dir.path().to_path_buf());
```

## Common Test Patterns

**Round-Trip Testing:**
- Convert to format, then convert back, verify equivalence
- Pattern from `tests/converter_tests.rs`:
```rust
#[test]
fn test_round_trip_conversion_cursor() {
    let converter = CursorConverter::new();
    let original_rule = create_test_rule();
    
    let cursor_format = converter.convert_to_tool_format(&original_rule).unwrap();
    let imported_rule = converter.convert_from_tool_format(&cursor_format).unwrap();
    
    assert_eq!(imported_rule.metadata.name, original_rule.metadata.name);
    assert_eq!(imported_rule.id, original_rule.id);
}
```

**Error Testing:**
- Verify errors are produced for invalid inputs
- Pattern:
```rust
#[test]
fn test_import_error_handling() {
    let converter = CursorConverter::new();
    let malformed_input = r#"---
invalid yaml: [
---
"#;
    
    let result = converter.convert_from_tool_format(malformed_input);
    assert!(result.is_err());
}
```

**Validation Testing:**
- Check severity levels separately
- Pattern from `tests/validation_tests.rs`:
```rust
let errors = validator.validate(&rule).unwrap();
let error_messages: Vec<_> = errors
    .iter()
    .filter(|e| matches!(e.severity, Severity::Error))
    .map(|e| &e.message)
    .collect();

assert!(error_messages.iter().any(|msg| msg.contains("Rule must have a name")));
```

**CLI Integration Testing:**
- Execute actual binary with `std::process::Command`
- Pattern from `tests/cli_integration_tests.rs`:
```rust
fn run_rulesify_command(args: &[&str], working_dir: &Path) -> Result<(String, String, i32), std::io::Error> {
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
    let (stdout, _stderr, exit_code) = run_rulesify_command(&["--help"], temp_dir.path())
        .expect("Failed to run rulesify --help");
    
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("Rulesify manages Universal Rule Files"));
}
```

## Coverage & Quality

**Coverage:**
- No formal coverage requirements enforced
- No coverage tool configured (could use `cargo-tarpaulin`)

**Test Types:**

**Unit Tests:**
- Embedded in source files under `#[cfg(test)]` module
- Test individual functions and logic: `test_merge_rules_priority_ordering`

**Integration Tests:**
- In `tests/` directory
- Test module interactions: `converter_tests.rs`, `storage_tests.rs`
- Test full workflows: `end_to_end_tests.rs`

**CLI Tests:**
- Binary integration tests in `tests/cli_integration_tests.rs`
- Test actual command execution and output
- Require compiled binary (`cargo build` first)

## Test Naming Patterns

**Test Function Names:**
- Pattern: `test_{module/feature}_{scenario}`
- Examples:
  - `test_cursor_converter_basic_conversion`
  - `test_file_store_save_and_load`
  - `test_content_validator_missing_name`
  - `test_cli_deploy_command`

**Descriptive Names:**
- Include what's being tested and the scenario
- Example: `test_merge_rules_tag_deduplication` tests merge rules with tag deduplication scenario

---

*Testing analysis: 2026-04-02*