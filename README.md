# Rulesify

Rulesify is a command-line tool (written in Rust) that provides **unified management of AI assistant rules** across multiple platforms. Create rules once in Universal Rule Format (URF), then deploy them everywhere.

## Supported AI Tools

- **Cursor** (`.cursor/rules/*.mdc` — Markdown with YAML frontmatter)
- **Cline** (`.clinerules/*.md` — Simple Markdown)
- **Claude Code** (`CLAUDE.md` — Markdown, hierarchical)
- **Goose** (`.goosehints` — Plain text)

## Features

- **Rule Management**: Create, edit, list, show, and delete rules
- **Multi-Tool Deployment**: Deploy rules to all supported AI tools
- **Import Functionality**: Import existing rules from any AI tool format
- **Validation System**: Comprehensive quality and format validation
- **Template System**: Create rules from built-in templates
- **Configuration Management**: Flexible storage and configuration options
- **Sync**: Synchronize deployed rules back to URF format
- **Round-Trip Integrity**: Import/export cycle is lossless and auto-validated
- **Unicode Support**: Handles international text
- **Comprehensive Testing**: 75 tests, 100% pass rate

## Installation

### One-liner (Recommended)

Run this command to install the latest pre-built binary:

```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash
```

- This script detects your OS/architecture, downloads the correct binary, installs it to `~/.local/bin`, and adds it to your `PATH` if needed.
- After installation, restart your shell or run:
  ```bash
  export PATH="$HOME/.local/bin:$PATH"
  ```

### Manual Installation (Fallback)

1. Download the latest binary for your OS/arch from [GitHub Releases](https://github.com/ihelio/rulesify/releases).
2. Move it to `~/.local/bin` (create the directory if it doesn't exist):
   ```bash
   mkdir -p ~/.local/bin
   mv rulesify-<os>-<arch> ~/.local/bin/rulesify
   chmod +x ~/.local/bin/rulesify
   ```
3. Add `~/.local/bin` to your `PATH` if not already present.

## Quick Start

### Basic Usage
```bash
# Create a new rule
rulesify rule new typescript-style

# Import existing rules
rulesify import --tool cursor .cursor/rules/my-rule.mdc
rulesify import --tool cline .clinerules/coding-standards.md

# Validate rules
rulesify validate --all
rulesify validate my-rule

# Deploy to all tools
rulesify deploy --all

# Deploy to specific tool
rulesify deploy --tool cursor --all
```

### Import Examples

Import rules from different AI tools:

```bash
# Import from Cursor
rulesify import --tool cursor .cursor/rules/python-style.mdc

# Import from Cline
rulesify import --tool cline .clinerules/react-patterns.md

# Import from Claude Code
rulesify import --tool claude-code CLAUDE.md

# Import from Goose
rulesify import --tool goose .goosehints
```

### Validation

Ensure your rules meet quality standards:

```bash
# Validate all rules
rulesify validate --all

# Validate specific rule
rulesify validate python-style

# Validation checks include:
# - Name and content requirements
# - File reference security
# - Format consistency
# - Pattern validation
```

## Universal Rule Format (URF)

Rulesify uses a **YAML-based Universal Rule Format** for internal storage. Each rule is a single `.urf.yaml` file. **Edit only these files**—generated tool files are Git-ignored. Each URF file starts with a fingerprint comment (`# sha256:...`) for integrity.

**Sample URF YAML:**
```yaml
# -------------------------------------------------------------
#  UNIVERSAL RULE FILE (URF) – SINGLE SOURCE OF TRUTH
#  Replace <placeholders> and delete comments after editing.
# -------------------------------------------------------------
id: typescript-style              # machine-safe slug, filled automatically
version: 1.0.0                    # bump when you make breaking changes
metadata:
  name: "TypeScript Style Guide"  # appears in exported Markdown H1
  description: |
    Coding standards for TypeScript projects
  tags: [typescript, style, linting]
  priority: 5                     # 1 (low) → 10 (high); used for ordering
  auto_apply: true                # if true, export uses alwaysApply in Cursor
content:
  - title: "Code Style"           # Markdown H2 in exports
    format: markdown              # or plaintext / code
    value: |-
      • Use consistent formatting...
      • Add more guidelines here
references:                       # optional list of @file references
  - @tsconfig.json
conditions:                       # optional glob patterns that trigger auto-attach
  - type: file_pattern
    value: '**/*.ts'
# Tool-specific overrides (ignored by other exporters)
tool_overrides:
  cursor:
    globs: [src/**/*.ts]
  cline:
    toggle_default: true
  claude-code: {}
  goose: {}
```

**Editing & Round-Trip Integrity:**
- Edit only `*.urf.yaml` files. Generated tool files are Git-ignored.
- Each URF file starts with a fingerprint comment (`# sha256:...`).
- Round-trip guarantee: `urf → tool file → urf` is lossless (auto-validated).
- Tool-specific quirks are isolated in `tool_overrides`.

## Command Reference

### Rule Management
- `rulesify rule new <name>` - Create a new rule
- `rulesify rule edit <name>` - Edit an existing rule
- `rulesify rule list` - List all rules
- `rulesify rule list -r <regex>` - List rules matching regex
- `rulesify rule show <name>` - Show rule details
- `rulesify rule delete <name>` - Delete a rule

### Import & Validation
- `rulesify import --tool <tool> <file>` - Import rule from AI tool format
- `rulesify validate [rule]` - Validate rule(s)
- `rulesify validate --all` - Validate all rules

### Deployment
- `rulesify deploy --all` - Deploy all rules to all tools
- `rulesify deploy --tool <tool> --all` - Deploy to specific tool
- `rulesify deploy --tool <tool> <rule>` - Deploy specific rule to tool
- `rulesify deploy --tool <tool> --rules <name> --dry-run` - Preview export

### Synchronization
- `rulesify sync [--dry-run]` - Synchronize rules across all tools
- `rulesify sync --rule <name>` - Sync a specific rule
- `rulesify sync --tool <tool>` - Sync for a specific tool

### Configuration
- `rulesify config show` - Show current configuration
- `rulesify config edit` - Edit configuration file
- `rulesify config set-storage <path>` - Change storage location
- `rulesify config set-editor <editor>` - Set default editor
- `rulesify config add-tool <tool>` - Add default deployment tool
- `rulesify config remove-tool <tool>` - Remove default deployment tool

## Example Usage Scenarios

### Author & Deploy a New Rule
```bash
rulesify rule new react-best-practices
rulesify validate react-best-practices
rulesify deploy --tool cursor --rules react-best-practices --dry-run
rulesify deploy --tool cursor --rules react-best-practices
rulesify deploy --tool cline --rules react-best-practices
```

### Import an Existing Cursor Rule
```bash
author_rule=.cursor/rules/coding-standards.mdc
rulesify import --tool cursor "$author_rule"
rulesify deploy --tool claude-code --rules coding-standards
```

### Share Rules Across the Team
```bash
rulesify rule export --name team-standards --format urf > team-standards.yaml
rulesify rule import --tool universal team-standards.yaml
rulesify validate --all
rulesify deploy --tool goose --rules team-standards
```

## Development

See [`DEVELOPMENT_PLAN_DETAILED.md`](./DEVELOPMENT_PLAN_DETAILED.md) for the complete development roadmap and architecture details.

### Project Structure
```
rulesify/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands/
│   ├── models/
│   ├── store/
│   ├── converters/
│   ├── templates/
│   ├── validation/
│   ├── sync/
│   └── utils/
├── templates/
├── tests/
└── docs/
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test converter_tests
cargo test --test validation_tests
```

### Test Coverage
- **75 total tests** across all modules
- **19 converter tests** (import/export functionality)
- **22 validation tests** (quality assurance)
- **8 CLI integration tests**
- **11 import tests**
- **4 end-to-end tests** (complete workflows)
- **5 storage tests** (persistence layer)
- **6 template tests** (rule generation)
- **Unicode and round-trip integrity tested**

## License

This project is licensed under the [MIT License](./LICENSE).

Copyright (c) 2024 Rulesify Contributors
