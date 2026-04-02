# Codebase Structure

**Analysis Date:** 2026-04-02

## Directory Layout

```
/Users/ihelio/code/rulesify/
├── Cargo.toml           # Package manifest with dependencies
├── Cargo.lock           # Dependency lockfile
├── README.md            # Project documentation and usage guide
├── CHANGELOG.md         # Version history and release notes
├── LICENSE              # MIT license
├── install.sh           # Installation script for releases
├── src/                 # Source code (main library)
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library entry point (module exports)
│   ├── cli/             # CLI command definitions and handlers
│   ├── models/          # Core data structures
│   ├── store/           # Persistence layer (trait + implementations)
│   ├── converters/      # Format conversion (trait + per-tool impls)
│   ├── templates/       # Template engine and builtin skeletons
│   ├── validation/      # Validation logic
│   ├── sync/            # Synchronization logic
│   └── utils/           # Shared utilities
├── templates/           # Template files (Handlebars-style)
├── tests/               # Integration tests
│   └── fixtures/        # Test fixture files
├── docs/                # Documentation (examples subdirectory)
├── .github/             # GitHub workflows
│   └── workflows/       # CI/CD workflows
├── target/              # Build artifacts (Rust compilation)
└── .planning/           # Planning documents (created by GSD)
```

## Directory Purposes

**`src/cli/`:**
- Purpose: Command-line interface implementation
- Contains: Clap-based command definitions, subcommand handlers
- Key files: `src/cli/mod.rs` (Cli struct, Commands enum), `src/cli/commands/*.rs` (per-command handlers)

**`src/models/`:**
- Purpose: Core data model definitions
- Contains: Rust structs and enums for rules, configuration, metadata
- Key files: `src/models/rule.rs` (UniversalRule, RuleMetadata, RuleContent), `src/models/config.rs` (GlobalConfig)

**`src/store/`:**
- Purpose: Rule persistence abstraction and implementation
- Contains: RuleStore trait, FileStore (YAML file storage), MemoryStore (testing)
- Key files: `src/store/mod.rs` (trait definition), `src/store/file_store.rs` (filesystem implementation)

**`src/converters/`:**
- Purpose: Format conversion between URF and tool-specific formats
- Contains: RuleConverter trait, per-tool converter implementations
- Key files: `src/converters/mod.rs` (trait), `src/converters/cursor.rs`, `src/converters/cline.rs`, `src/converters/claude_code.rs`, `src/converters/goose.rs`

**`src/validation/`:**
- Purpose: Rule quality and format validation
- Contains: Validator trait, ContentValidator, FormatValidator
- Key files: `src/validation/mod.rs` (trait, ValidationError, Severity), `src/validation/content_validator.rs`

**`src/sync/`:**
- Purpose: Synchronization of deployed rules back to URF
- Contains: Synchronizer struct
- Key files: `src/sync/mod.rs`, `src/sync/synchronizer.rs`

**`src/utils/`:**
- Purpose: Shared utility functions
- Contains: Config loading, filesystem helpers, rule ID handling
- Key files: `src/utils/config.rs` (load/save GlobalConfig), `src/utils/rule_id.rs` (sanitize, embed, extract rule IDs)

**`src/templates/`:**
- Purpose: Rule skeleton generation
- Contains: TemplateEngine, builtin skeleton generator
- Key files: `src/templates/mod.rs`, `src/templates/engine.rs`, `src/templates/builtin.rs`

**`tests/`:**
- Purpose: Integration and end-to-end tests
- Contains: Per-module test files, test fixtures
- Key files: `tests/converter_tests.rs`, `tests/validation_tests.rs`, `tests/cli_integration_tests.rs`, `tests/import_tests.rs`

**`templates/`:**
- Purpose: Template source files
- Contains: Handlebars-style template files for rule generation
- Key files: `templates/typescript-style.hbs`

**`.github/workflows/`:**
- Purpose: CI/CD automation
- Contains: GitHub Actions workflow definitions
- Key files: `.github/workflows/tests.yml`, `.github/workflows/release.yml`, `.github/workflows/releases.yml`

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point - initializes logging, parses args, executes commands
- `src/lib.rs`: Library entry point - exports all public modules

**Configuration:**
- `Cargo.toml`: Package manifest with dependencies and metadata
- `~/.rulesify/config.yaml`: User configuration file (runtime, not in repo)

**Core Logic:**
- `src/models/rule.rs`: UniversalRule and related types - the central data model
- `src/store/file_store.rs`: FileStore implementation - rule persistence
- `src/converters/cursor.rs`: Cursor format conversion (most complex converter)
- `src/cli/commands/deploy.rs`: Deploy command - handles single and multi-rule deployment with merging

**Testing:**
- `tests/converter_tests.rs`: 19 converter tests
- `tests/validation_tests.rs`: 22 validation tests
- `tests/cli_integration_tests.rs`: 10+ CLI tests
- `tests/fixtures/`: Test fixture files (URF samples)

## Naming Conventions

**Files:**
- Rust source: `snake_case.rs` (e.g., `file_store.rs`, `content_validator.rs`)
- Module directories: `snake_case/` (e.g., `cli/`, `models/`, `converters/`)
- Test files: `*_tests.rs` (e.g., `converter_tests.rs`, `validation_tests.rs`)
- URF files: `{rule-id}.urf.yaml` (e.g., `typescript-style.urf.yaml`)
- Template files: `snake_case.hbs` (e.g., `typescript-style.hbs`)

**Directories:**
- Module directories: `snake_case` matching Rust module names
- Test fixture directory: `fixtures/`

**Rust Identifiers:**
- Structs: `PascalCase` (e.g., `UniversalRule`, `FileStore`, `CursorConverter`)
- Traits: `PascalCase` (e.g., `RuleStore`, `RuleConverter`, `Validator`)
- Functions: `snake_case` (e.g., `load_rule`, `convert_to_tool_format`, `sanitize_rule_id`)
- Enums: `PascalCase` for type, `PascalCase` for variants (e.g., `Severity::Error`, `Commands::Deploy`)

## Where to Add New Code

**New CLI Command:**
1. Add command variant to `Commands` enum in `src/cli/mod.rs`
2. Create handler file in `src/cli/commands/{command_name}.rs`
3. Add module export in `src/cli/commands/mod.rs`
4. Add dispatch logic in `Cli::execute()` method in `src/cli/mod.rs`

**New Tool Converter:**
1. Create converter file in `src/converters/{tool_name}.rs`
2. Implement `RuleConverter` trait
3. Add module export in `src/converters/mod.rs`
4. Register converter in `get_converter()` function in `src/cli/commands/deploy.rs`
5. Add tool to supported tools list in README and CLI help text

**New Validator:**
1. Create validator file in `src/validation/{validator_name}.rs`
2. Implement `Validator` trait
3. Add module export in `src/validation/mod.rs`

**New Model/Type:**
1. Add struct/enum to appropriate file in `src/models/`
2. Add module export in `src/models/mod.rs` if new file
3. Ensure serde `Serialize/Deserialize` derives for persistence

**New Utility Function:**
1. Add to appropriate file in `src/utils/`
2. Re-export in `src/utils/mod.rs` if broadly useful

**New Tests:**
- Integration tests: Add to `tests/{module}_tests.rs`
- Unit tests: Add `#[cfg(test)] mod tests` block in source file

## Special Directories

**`target/`:**
- Purpose: Rust build artifacts and compilation output
- Generated: Yes (by cargo build/test)
- Committed: No (in .gitignore)

**`.planning/`:**
- Purpose: GSD planning documents
- Generated: Yes (by GSD commands)
- Committed: Yes (intended to be versioned)

**`tests/fixtures/`:**
- Purpose: Test fixture files for integration tests
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-04-02*