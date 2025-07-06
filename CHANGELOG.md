# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
