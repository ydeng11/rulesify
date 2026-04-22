# Technical Concerns

## Known Issues

**Duplicate Parsing Logic:**
- `claude_code.rs` and `cline.rs` contain identical `parse_*_format` functions
- Files: `src/converters/claude_code.rs:81-160`, `src/converters/cline.rs:81-160`
- Impact: Code maintenance burden, fixes must be applied twice
- Fix approach: Extract common markdown parsing into shared utility module

**Duplicate get_converter Function:**
- Same function exists in 3 separate command files
- Files: `src/cli/commands/deploy.rs:181-196`, `src/cli/commands/sync.rs:109-120`, `src/cli/commands/import.rs:116-127`
- Impact: Maintenance overhead, inconsistency risk
- Fix approach: Move to `src/converters/mod.rs` as public utility function

## Technical Debt

**Large File Complexity:**
- Issue: `deploy.rs` is 672 lines with mixed concerns
- Files: `src/cli/commands/deploy.rs`
- Impact: Difficult to navigate, test, and maintain
- Contains: Deploy logic, merge logic, interactive prompts, and unit tests
- Fix approach: Split into deploy.rs, merge.rs, and move tests to tests/

**Deprecated Field Still Present:**
- Issue: `auto_apply` field marked deprecated but still generated in output
- Files: `src/converters/cursor.rs:211-215`
- Impact: Confusion for users, unnecessary data in URF files
- Current mitigation: Both `apply_mode` and `auto_apply` written for backwards compatibility
- Fix approach: Add migration guide, phase out `auto_apply` in future version

**Regex-Based YAML Field Updates:**
- Issue: Sync command uses regex to update YAML fields, fragile with unusual formatting
- Files: `src/cli/commands/sync.rs:316-339`
- Impact: Could corrupt YAML files with edge-case formatting
- Safe modification: Ensure YAML is normalized before sync operations
- Fix approach: Use proper YAML parser (serde_yaml) for field modifications instead of regex

## Security Considerations

**Path Traversal Warnings:**
- Risk: File references with ".." could potentially access files outside project
- Files: `src/validation/content_validator.rs:155-161`
- Current mitigation: Warning issued during validation, not blocked
- Recommendations: Consider blocking or sandboxing file references with ".."

**Install Script Binary Verification:**
- Risk: Install script downloads binary without checksum/signature verification
- Files: `install.sh:140-168`
- Current mitigation: Uses GitHub releases which have some trust
- Recommendations: Add SHA256 checksum verification for downloaded binaries

**Editor Command Execution:**
- Risk: Editor command executed from user config without validation
- Files: `src/cli/commands/rule.rs:82-87`, `src/cli/commands/import.rs:102-106`
- Current mitigation: User must configure editor explicitly
- Recommendations: Validate editor command exists before execution

## Performance Bottlenecks

**Multiple Rule Loading:**
- Problem: Deploy with `--all` loads each rule individually
- Files: `src/cli/commands/deploy.rs:45-46`
- Cause: `store.list_rules()` then `store.load_rule()` for each
- Improvement path: Add batch loading method to RuleStore trait

**Regex Compilation in Validators:**
- Problem: Regex patterns compiled on every validation call
- Files: `src/validation/format_validator.rs:34-35`, `src/validation/format_validator.rs:53`
- Cause: `Regex::new()` called inside `validate()` method
- Improvement path: Use `lazy_static!` or `once_cell` for compiled patterns

## Fragile Areas

**Interactive Input Handling:**
- Files: `src/cli/commands/deploy.rs:106-123`, `src/cli/commands/rule.rs:260-272`
- Why fragile: Depends on stdin availability, fails in non-interactive environments
- Safe modification: Add `--force` or `--no-confirm` flags for CI/automation
- Test coverage: Integration tests cannot test interactive prompts

**YAML Frontmatter Parsing:**
- Files: `src/converters/cursor.rs:242-282`
- Why fragile: Manual line-by-line parsing, edge cases with malformed YAML
- Safe modification: Add more error handling for edge cases
- Test coverage: Has tests for malformed YAML but could use more edge cases

**Rule ID Fallback Logic:**
- Files: `src/utils/rule_id.rs:122-154`
- Why fragile: Multiple fallback methods, could produce unexpected IDs
- Safe modification: Document behavior clearly, add logging for which fallback used
- Test coverage: Has unit tests but real-world edge cases may differ

## Scaling Limits

**File-Based Storage:**
- Current capacity: Designed for tens of rules, not hundreds
- Limit: Each rule is a separate YAML file, no indexing
- Files: `src/store/file_store.rs`
- Scaling path: Consider database or indexed storage for large rule collections

**Single-Threaded Processing:**
- Current capacity: Rules processed sequentially
- Limit: Large rule counts will have noticeable latency
- Files: `src/cli/commands/deploy.rs:84-167`
- Scaling path: Add parallel processing with rayon for batch operations

## Dependencies at Risk

**serde_yaml Maintenance:**
- Risk: serde_yaml is less actively maintained than serde_json
- Impact: Potential future compatibility issues
- Files: `Cargo.toml:13`
- Migration plan: Monitor serde_yaml status, consider yaml-rust2 as alternative

**dirs Package:**
- Risk: Small dependency, single-purpose
- Impact: If unmaintained, home directory detection could fail
- Files: `Cargo.toml:20`, `src/utils/config.rs:7-10`
- Migration plan: Could implement home directory detection directly

## Missing Critical Features

**Batch Operations:**
- Problem: No bulk import/export functionality
- Blocks: Users with large existing rule collections
- Files: CLI commands only support single operations

**Rule Versioning/Diff:**
- Problem: No way to track rule changes over time
- Blocks: Auditing what changed in rules
- Files: `src/models/rule.rs:40` has version field but no history

**Rule Templates:**
- Problem: Only one skeleton template available
- Blocks: Quick creation of specialized rule types
- Files: `src/templates/builtin.rs:3-114`

## Test Coverage Gaps

**Error Path Testing:**
- What's not tested: File permission errors, disk full scenarios, corrupt YAML recovery
- Files: All test files focus on success paths
- Risk: Application may crash or behave unexpectedly in error conditions
- Priority: Medium

**Concurrency Testing:**
- What's not tested: Multiple simultaneous deploy/sync operations
- Files: No tests for concurrent access to rule store
- Risk: Race conditions could corrupt rule files
- Priority: High if used in CI/CD pipelines

**Binary Integration Tests:**
- What's not tested: CLI tests require pre-built binary
- Files: `tests/cli_integration_tests.rs:6-24` panics if binary missing
- Risk: Tests fail in environments without pre-built binary
- Priority: Medium

**Coverage Measurement:**
- What's not tested: No coverage reporting in CI
- Files: `.github/workflows/tests.yml` has no coverage step
- Risk: Unknown coverage percentage, potential untested code paths
- Priority: Low

---

*Concerns audit: 2026-04-02*