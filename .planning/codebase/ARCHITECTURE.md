# Architecture

**Analysis Date:** 2026-04-02

## Pattern Overview

**Overall:** Layered CLI Application with Plugin-style Converters

**Key Characteristics:**
- Command-line interface using Clap for argument parsing and subcommand routing
- Trait-based abstraction for storage (RuleStore) and format conversion (RuleConverter)
- Universal Rule Format (URF) as canonical internal representation
- Bidirectional conversion between URF and tool-specific formats (Cursor, Cline, Claude Code, Goose)
- YAML-based persistence with fingerprinting for integrity tracking

## Layers

**CLI Layer:**
- Purpose: Handle command parsing, routing, and user interaction
- Location: `src/cli/` and `src/main.rs`
- Contains: Command definitions (Clap structs), subcommand handlers, user prompts, output formatting
- Depends on: Store layer, Converter layer, Utils layer
- Used by: User via terminal

**Store Layer:**
- Purpose: Persist and retrieve Universal Rules from filesystem
- Location: `src/store/`
- Contains: `RuleStore` trait, `FileStore` implementation, `MemoryStore` for testing
- Depends on: Models layer, serde_yaml for serialization
- Used by: CLI commands, Converters for loading rules

**Converter Layer:**
- Purpose: Transform between URF and tool-specific formats
- Location: `src/converters/`
- Contains: `RuleConverter` trait, per-tool implementations (Cursor, Cline, Claude Code, Goose)
- Depends on: Models layer, regex for parsing
- Used by: Deploy command, Import command, Sync command

**Models Layer:**
- Purpose: Define core data structures and types
- Location: `src/models/`
- Contains: `UniversalRule`, `RuleMetadata`, `RuleContent`, `GlobalConfig`
- Depends on: serde for serialization, chrono for timestamps
- Used by: All other layers

**Validation Layer:**
- Purpose: Validate rule content and structure for quality
- Location: `src/validation/`
- Contains: `Validator` trait, `ContentValidator`, `FormatValidator`
- Depends on: Models layer
- Used by: Validate command

**Sync Layer:**
- Purpose: Synchronize deployed tool files back to URF format
- Location: `src/sync/`
- Contains: `Synchronizer` struct (partial implementation)
- Depends on: Converter layer, Store layer
- Used by: Sync command

**Utils Layer:**
- Purpose: Provide shared utilities for configuration, filesystem, and rule ID handling
- Location: `src/utils/`
- Contains: Config loading/saving, filesystem helpers, rule ID sanitization/embedding
- Depends on: Models layer, dirs crate for home directory
- Used by: All CLI commands

**Templates Layer:**
- Purpose: Generate skeleton rule files from templates
- Location: `src/templates/`
- Contains: `TemplateEngine`, builtin skeleton generator
- Depends on: None (standalone)
- Used by: Rule new command

## Data Flow

**Create Rule Flow:**
1. User runs `rulesify rule new <name>`
2. CLI sanitizes name to valid rule ID via `src/utils/rule_id.rs`
3. Template engine generates skeleton YAML
4. FileStore writes to `{rules_directory}/{rule_id}.urf.yaml`
5. Editor opens file if configured

**Deploy Rule Flow:**
1. User runs `rulesify deploy --tool <tool> --rule <rule>`
2. CLI loads config to determine rules directory
3. FileStore loads `UniversalRule` from YAML file
4. Converter transforms `UniversalRule` to tool-specific format (e.g., YAML frontmatter + Markdown for Cursor)
5. Converter writes to tool-specific path (e.g., `.cursor/rules/{rule}.mdc`)
6. Rule ID embedded as HTML comment for tracking

**Import Rule Flow:**
1. User runs `rulesify import --tool <tool> <file>`
2. CLI reads tool-specific file content
3. Converter parses content and creates `UniversalRule`
4. Rule ID determined via fallback hierarchy (embedded comment > filename > rule name > timestamp)
5. FileStore saves as `{rule_id}.urf.yaml`

**Sync Rule Flow:**
1. User runs `rulesify sync`
2. Synchronizer reads deployed tool files
3. Converter parses tool format back to `UniversalRule`
4. FileStore compares with existing URF files
5. Updates URF files when differences detected

**State Management:**
- Rules stored as individual YAML files in configurable directory
- Config stored in `~/.rulesify/config.yaml`
- No database; filesystem is the persistence layer
- In-memory only during command execution

## Key Abstractions

**RuleStore Trait:**
- Purpose: Abstract storage operations for rules
- Examples: `src/store/mod.rs`, `src/store/file_store.rs`
- Pattern: Repository pattern with CRUD operations
- Methods: `load_rule`, `save_rule`, `list_rules`, `delete_rule`

**RuleConverter Trait:**
- Purpose: Abstract bidirectional format conversion
- Examples: `src/converters/mod.rs`, `src/converters/cursor.rs`, `src/converters/cline.rs`
- Pattern: Strategy pattern - each tool has its own converter implementation
- Methods: `convert_to_tool_format`, `convert_from_tool_format`, `get_deployment_path`, `get_file_extension`

**Validator Trait:**
- Purpose: Abstract validation logic for rules
- Examples: `src/validation/mod.rs`, `src/validation/content_validator.rs`
- Pattern: Validator pattern with severity levels (Error, Warning, Info)
- Method: `validate` returning `Vec<ValidationError>`

**UniversalRule Struct:**
- Purpose: Canonical internal representation of a rule
- Examples: `src/models/rule.rs`
- Pattern: Central data model with tool-specific overrides via HashMap
- Fields: `id`, `version`, `metadata`, `content`, `references`, `conditions`, `tool_overrides`

## Entry Points

**Binary Entry Point:**
- Location: `src/main.rs`
- Triggers: User executing `rulesify` command
- Responsibilities: Initialize logging, parse CLI args, dispatch to subcommand handlers, handle errors

**Library Entry Point:**
- Location: `src/lib.rs`
- Triggers: Other crates linking rulesify as library
- Responsibilities: Export all public modules (cli, models, store, converters, etc.)

## Error Handling

**Strategy:** anyhow Result type with error chaining

**Patterns:**
- Use `anyhow::Result` for all fallible operations
- Use `.context()` to add descriptive error messages
- Use `thiserror` for custom error types (not heavily used)
- Error chain logged to stderr with "Caused by" cascade in main.rs
- Exit code 1 on any error

## Cross-Cutting Concerns

**Logging:** env_logger with debug/info/error levels; `--verbose` flag enables debug output

**Validation:** ContentValidator checks name, description, content sections, tags, references, conditions; FormatValidator handles format-specific validation

**Configuration:** YAML-based config in `~/.rulesify/config.yaml`; supports custom config path via `--config` flag; defaults: rules in `~/.rulesify/rules/`, editor from `$EDITOR`, default tools `[cursor, cline]`

**Rule ID Management:** Centralized in `src/utils/rule_id.rs`; sanitization ensures lowercase-hyphen format; embedding in HTML comments for tracking; fallback hierarchy for import

---

*Architecture analysis: 2026-04-02*