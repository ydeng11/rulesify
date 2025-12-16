# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-12-16

### Fixed
- **URF parsing for `references`**: accept both string references (e.g. `- https://...`) and object references (e.g. `- path: https://...`).
- **URF parsing for `conditions`**: correctly deserialize condition entries via a tagged `type` field (e.g. `type: file_pattern`).

### Testing
- Added regression tests + fixture to prevent parsing regressions for real-world URF files.
- Reduced CI flakiness by serializing tests that change the process working directory.

## [0.3.0] - 2025-01-09

### Added
- **🚀 Shell Completion Support**: Interactive tab completion for enhanced CLI user experience
  - **Universal Shell Support**: Completion scripts for bash, zsh, fish, and PowerShell
  - **Intelligent Command Completion**: Context-aware suggestions for commands, subcommands, and options
  - **Automated Installation**: `install.sh` script automatically sets up completion for detected shell
  - **Manual Generation**: `rulesify completion <shell>` command for custom installation
  - **Comprehensive Testing**: Full test coverage for all shell types and error scenarios

### Enhanced
- **Installation Script**: Enhanced with automatic shell completion setup
  - Detects user's shell (bash, zsh, fish)
  - Installs completion scripts in appropriate system locations
  - Configures shell profiles for immediate availability
  - Provides fallback options for non-standard configurations
- **User Experience**: Significantly improved CLI usability with discoverable commands and options
- **Error Handling**: Proper validation and error messages for unsupported shells

### Technical Improvements
- Added `clap_complete` dependency for robust completion generation
- Implemented `completion.rs` command module with clean architecture
- Updated CLI help text to include completion examples
- Enhanced integration tests with comprehensive completion validation
- Zero breaking changes - fully backward compatible

### Documentation
- Updated README.md with completion installation instructions
- Added completion usage examples in CLI help text
- Documented shell-specific installation paths and procedures

## [0.2.0] - 2025-01-09

### Added
- **🚀 Multi-Rule Merging Support**: Enhanced deploy command with intelligent rule combination functionality
  - **Comma-Separated Rule Deployment**: Deploy multiple rules using `--rule "rule1,rule2,rule3"` syntax
  - **Priority-Based Merging**: Rules automatically sorted and merged by priority (10 → 1, highest first)
  - **Interactive User Experience**:
    - Merge preview showing rules in priority order with combined metadata
    - User-prompted merged rule ID with automatic sanitization
    - Conflict detection and overwrite confirmation
  - **Smart Metadata Combination**:
    - Name: Uses highest priority rule's name
    - Description: Concatenates all descriptions with separators (`\n\n---\n\n`)
    - Tags: Deduplicates across all rules while preserving priority order
    - Tool Overrides: Uses complete overrides from highest priority rule
  - **Universal Compatibility**: Works with all supported AI tools (cursor, cline, claude-code, goose)

### Enhanced
- **Deploy Command**: Extended to support both single and multiple rule deployment
  - Backward compatible: existing single-rule deployments work identically
  - Enhanced validation: all specified rules validated before proceeding
  - Improved error handling and user feedback
- **CLI Help**: Updated command descriptions to reflect new multi-rule capabilities
- **Documentation**: Comprehensive examples and usage scenarios for multi-rule workflows

### Added Functions
- `merge_rules()`: Core merging logic with priority-based rule combination
- `prompt_for_merged_rule_id()`: Interactive user input with validation and sanitization
- `show_merge_preview()`: User-friendly preview of merge operation showing priority order
- `deploy_merged_rule()`: Specialized deployment logic for merged rules

### Testing
- **6 New Unit Tests**: Complete coverage of merge logic (priority ordering, tag deduplication, content combination, description concatenation, edge cases)
- **2 New Integration Tests**: CLI behavior validation and error handling for multi-rule scenarios
- **Zero Breaking Changes**: All existing tests continue to pass (78+ total tests)

### Technical Improvements
- Enhanced rule parsing to handle comma-separated input with validation
- Improved user interaction patterns with clear previews and confirmations
- Better error messages and user guidance throughout the merge process
- Comprehensive test coverage ensuring reliability and preventing regressions

### Use Cases Enabled
- **Single-File AI Tools**: Perfect solution for claude-code (CLAUDE.md) and goose (.goosehints) that support only one rule file
- **Comprehensive Rule Sets**: Combine multiple specialized rules (e.g., typescript-style + react-patterns + testing-standards) into unified guidance
- **Team Workflows**: Merge individual contributor rules into team-wide standards
- **Tool Migration**: Easily consolidate rules when switching between AI tools

## [0.1.6] - 2025-01-09

### Added
- **Unified Rule ID Sanitization System**: New comprehensive rule ID management with consistent sanitization across all operations
  - Added `src/utils/rule_id.rs` module with unified sanitization logic
  - Rule IDs now consistently use lowercase, hyphens, and alphanumeric characters only
  - Enforced length limits (2-50 characters) with proper validation
  - Unified sanitization across rule creation, import, and sync operations

### Enhanced
- **Improved Sync Operations with HTML Comment Tracking**: Enhanced rule ID preservation during sync operations
  - Added HTML comment embedding (`<!-- rulesify-id: {rule_id} -->`) for better rule tracking
  - Sync operations now preserve original rule IDs more reliably using filename-based fallback hierarchy
  - Improved rule ID determination with priority-based fallback system (embedded ID → filename → rule name)
  - Better handling of edge cases where rule names differ from filenames

### Fixed
- **Claude Code Converter Naming**: Resolved naming inconsistencies in Claude Code rule handling
- **Sync Test Race Conditions**: Fixed intermittent test failures in CI/CD by improving test isolation
- **Rule ID Generation**: Consistent rule ID generation across all converters and commands

### Technical Improvements
- Added comprehensive test coverage for rule ID sanitization and validation
- Enhanced error handling for invalid rule IDs and file references
- Improved code organization with centralized rule ID management utilities
- Better documentation and inline comments for rule ID handling logic

## [0.1.5] - 2024-12-19

### Added
- **NEW**: Full support for all 4 Cursor rule application modes:
  - `always`: Apply to every chat and cmd-k session
  - `intelligent`: When Agent decides it's relevant (RECOMMENDED)
  - `specific_files`: When file matches specified patterns
  - `manual`: Only when @-mentioned by user
- New `apply_mode` field in `tool_overrides.cursor` with full backwards compatibility
- Comprehensive test suite for all application modes and round-trip conversion
- Template now includes documentation for all 4 application modes

### Fixed
- **CRITICAL**: Cursor "Apply Intelligently" mode now works correctly by placing rule description in the `description` field instead of `notes`
  - This ensures Cursor's AI can properly determine when rules are relevant
  - Rule name is now in `notes` field for reference
- Cursor frontmatter glob patterns are now properly quoted to prevent YAML parsing errors
- Globs now only output when `apply_mode` is "specific_files" (cleaner generated files)
- Improved logic for determining application mode during import/export

### Changed
- Updated all existing URF files to use new `apply_mode` structure (fully backwards compatible)
- CLI commands now show "Auto-apply (Cursor)" with mode-specific information
- Template defaults to `intelligent` mode (recommended best practice)

## [0.1.3] - 2024-12-19

### Added
- **Enhanced Build and Release Management**: Added comprehensive build and release management guidelines to Rust programming rule
  - Always use `cargo` to manage Rust projects for dependencies, builds, and toolchain management
  - Always use `rustfmt` (via `cargo fmt`) to format `.rs` files consistently
  - Always use `cargo check` to verify project compilation without generating executables
  - Always use `cargo build` to create executables in `target/debug/` for development and testing
  - Always use `cargo build --release` to build optimized executables for production release

### Fixed
- **GitHub Actions Test Failures**: Fixed race condition in sync tests that caused failures in CI/CD pipeline
  - Added `--test-threads=1` to GitHub Actions workflow to prevent race conditions
  - Fixed header comment extraction bug in `fallback_to_complete_update` function
  - All 78 tests now pass reliably in both local and CI environments

### Changed
- Updated Rust programming rule with expanded build management best practices
- Enhanced rule template with comprehensive documentation and usage examples
- Improved URF file structure with clearer comments and instructions

## [0.1.2] - 2024-12-19

### Fixed
- **Smart Sync with Comment Preservation**: Sync now preserves comments and formatting in URF files while updating only changed content
  - Resolves issue where sync was removing helpful comments from URF files
  - Example: Header comments like `# =============================================================================` are now preserved
  - Only modified fields (metadata, content, references, conditions) are updated
  - Maintains URF file readability and documentation

### Added
- **Selective Field Updating**: Sync now uses intelligent field-by-field updates for metadata changes
  - Updates only changed fields: name, description, tags, priority
  - Preserves original file structure and formatting
  - Falls back to complete update for structural changes while preserving header comments

### Changed
- Enhanced sync algorithm to be more precise and preserve manual formatting
- Improved user experience by maintaining URF file readability
- Updated version from 0.1.1 to 0.1.2

### Technical Details
- Implemented `update_urf_file_selectively()` function with regex-based field updating
- Added `fallback_to_complete_update()` for structural changes while preserving header comments
- Modified sync logic to detect and handle different types of changes appropriately
- All 78 tests continue to pass

## [0.1.1] - 2024-12-19

### Fixed
- **Sync Command**: Fixed critical issue where `rulesify sync` was creating new URF files with generated IDs instead of updating existing ones
  - Sync now preserves the original filename-based rule ID
  - Example: `rulesify sync --tool cursor --rule rust-programming` now correctly updates `rust-programming.urf.yaml` instead of creating `core-principles.urf.yaml`
  - Added warning message when no existing URF file is found, suggesting users use `rulesify import` instead

### Added
- **Version Flag**: Added `--version` flag to display the current version
- **Enhanced Install Script**: Improved `install.sh` with better update handling
  - Shows current vs latest version comparison
  - Provides clear feedback for fresh installs vs updates
  - Verifies installation success
  - Handles cases where user already has the latest version

### Testing
- Added comprehensive sync functionality tests (3 new tests)
- All 78 tests now pass (75 existing + 3 new sync tests)
- Tests verify sync preserves rule IDs and handles edge cases correctly

### Technical Details
- Modified `src/cli/commands/sync.rs` to override converter-generated IDs with filename-based IDs
- Added version support using clap's built-in version handling
- Enhanced error handling and user feedback in sync operations

## [0.1.0] - 2024-12-19

### Added
- Initial release of Rulesify
- Support for 4 AI tools: Cursor, Cline, Claude Code, Goose
- Universal Rule Format (URF) for managing rules across tools
- Rule management commands: create, edit, list, show, delete
- Deploy rules to multiple AI tools simultaneously
- Import existing rules from AI tool formats
- Comprehensive validation system
- Configuration management
- Template system for rule creation
- 75 comprehensive tests with 100% pass rate
