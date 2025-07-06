# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2024-12-19

### Added
- **Enhanced Build and Release Management**: Added comprehensive build and release management guidelines to Rust programming rule
  - Always use `cargo` to manage Rust projects for dependencies, builds, and toolchain management
  - Always use `rustfmt` (via `cargo fmt`) to format `.rs` files consistently
  - Always use `cargo check` to verify project compilation without generating executables
  - Always use `cargo build` to create executables in `target/debug/` for development and testing
  - Always use `cargo build --release` to build optimized executables for production release

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
  - Updates only changed fields: name, description, tags, priority, auto_apply
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
