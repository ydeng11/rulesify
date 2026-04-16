# Rulesify

Rulesify is a command-line tool (written in Rust) that provides **unified management of AI assistant rules** across multiple platforms. Create rules once in Universal Rule Format (URF), then deploy them everywhere.

## Supported AI Tools

- **Cursor** (`.cursor/rules/*.mdc` — Markdown with YAML frontmatter)
- **Cline** (`.clinerules/*.md` — Simple Markdown files)
- **Claude Code** (`<rule-name>.md` — Markdown files in project root)
- **Goose** (`<rule-name>.goosehints` — Plain text files in project root)

**File Generation Details:**
- Generated tool files are automatically Git-ignored
- Only edit URF files (`.urf.yaml`) - tool files are regenerated on deployment
- Each tool has specific format requirements that are handled automatically
- Round-trip integrity: URF → tool file → URF maintains all information

## Features

- **Rule Management**: Create, edit, list, show, and delete rules
- **Multi-Tool Deployment**: Deploy rules to all supported AI tools
- **Skills Registry**: Browse and install AI agent skills from curated GitHub repos
- **LLM Classification**: Automatic domain and tag assignment for skills via OpenRouter
- **Import Functionality**: Import existing rules from any AI tool format
- **Validation System**: Comprehensive quality and format validation
- **Template System**: Create rules from built-in templates
- **Configuration Management**: Flexible storage and configuration options
- **Sync**: Synchronize deployed rules back to URF format
- **Round-Trip Integrity**: Import/export cycle is lossless and auto-validated
- **Unicode Support**: Handles international text
- **Comprehensive Testing**: 38 tests, 100% pass rate

## Skills Registry

Rulesify includes a curated catalog of AI agent skills from trusted GitHub repositories. Skills are pre-built workflows and methodologies that enhance AI assistant capabilities.

### Skill Sources

Skills are fetched from:
- **anthropics/skills** - Anthropic's official skill collection
- **openai/skills** - OpenAI's curated and system skills
- **mattpocock/skills** - Matt Pocock's development skills
- **MiniMax-AI/skills** - MiniMax's skill collection

### LLM-Based Classification

Skills are automatically classified using LLM into:

**10 Predefined Domains:**
- `planning-and-workflows` - Project planning, task management, workflows
- `development` - Code development, SDKs, application building
- `design-and-media` - UI design, visual arts, creative work
- `documentation` - Document creation, formatting, content management
- `data-and-research` - Data analysis, research, information synthesis
- `testing-and-debugging` - Testing, debugging, error handling
- `deployment-and-infrastructure` - Cloud deployment, infrastructure, hosting
- `integrations-and-tools` - API integrations, tool building, MCP servers
- `collaboration-and-communication` - Team communication, meeting tools
- `security-and-privacy` - Security reviews, threat modeling, privacy

**Tags:** Each skill has up to 3 relevant tags describing its capabilities.

### Registry Format

Each skill entry includes metadata for discovery:

```toml
[skills.tdd]
name = "test-driven-development"
description = "Write tests before implementation code using TDD methodology"
source_url = "https://github.com/mattpocock/skills/tree/main/tdd"
stars = 1500
context_size = 2400
domain = "development"
last_updated = "2026-04-15"
tags = ["testing", "tdd", "best-practices"]
install_action = { type = "copy", value = { folder = "tdd" } }
```

### Install Actions

Skills have two installation types:

- **Copy** (`type = "copy"`): Skills stored as folders - rulesify copies the entire skill folder
- **Command** (`type = "command"`): Complex skills - custom install command (agent executes)

### Browse Skills

```bash
# List all available skills
rulesify skill list

# Filter by domain
rulesify skill list --domain development

# Filter by tags
rulesify skill list --tags testing,debugging

# Show skill details
rulesify skill show tdd
```

### Install Skills

```bash
# Install a skill (for simple skills)
rulesify skill add tdd

# For complex skills, rulesify outputs the install command
rulesify skill add gsd
# Output: Run: git clone https://github.com/gsd-build/get-shit-done ~/.agents/skills/gsd
```

### Update Registry

The registry updates weekly via GitHub Actions. To manually refresh with LLM classification:

```bash
# Requires both GitHub and OpenRouter tokens
GITHUB_TOKEN=your_token OPENROUTER_API_KEY=your_key cargo run --bin update-registry

# Force re-classification of all skills (ignore cache)
cargo run --bin update-registry -- --force

# Verbose mode for debugging
cargo run --bin update-registry -- --force -v

# Use custom model (default: anthropic/claude-3.5-haiku)
OPENROUTER_MODEL=openai/gpt-4o-mini cargo run --bin update-registry -- --force
```

**Classification Cache:**
- Domain and tags are cached in registry.toml
- Only new/unclassified skills call the LLM
- Use `--force` to re-classify all skills

## Installation

### One-liner (Recommended)

Run this command to install the latest stable release:

```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash
```

- This script detects your OS/architecture, downloads the correct binary, installs it to `~/.local/bin`, and adds it to your `PATH` if needed.
- After installation, restart your shell or run:
  ```bash
  export PATH="$HOME/.local/bin:$PATH"
  ```

### Installing Edge Releases

Edge releases are pre-release builds from the latest merged commit on the default branch. They allow you to test the newest changes without waiting for a production tag.

**Install latest edge release (moving tag `edge`):**
```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash -s -- --edge
```

**Install a specific edge release by commit SHA (tag `edge-<sha>`):**
```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash -s -- --edge abc1234
```

**Install specific version tag:**
```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash -s -- v0.3.0
```

**Show help:**
```bash
curl -sSL https://github.com/ihelio/rulesify/releases/latest/download/install.sh | bash -s -- --help
```

**Note:** Edge releases are tagged as `edge` (latest) and `edge-<sha>` (immutable). They are pre-release builds and may be unstable. Use stable releases for production.

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

# Import existing rules from different tools
rulesify import --tool cursor .cursor/rules/my-rule.mdc
rulesify import --tool cline .clinerules/coding-standards.md

# Validate rules for quality and compliance
rulesify validate --all
rulesify validate typescript-style

# Deploy to all configured tools
rulesify deploy --all

# Deploy to specific tool
rulesify deploy --tool cursor --all
rulesify deploy --tool cline --rule typescript-style

# Sync deployed rules back to URF (preview first)
rulesify sync --dry-run
rulesify sync
```

### Synchronization Examples

**Sync reads deployed tool files and updates URF files when changes are detected.**

```bash
# Synchronize all deployed rules back to URF format
rulesify sync

# Preview changes without applying them
rulesify sync --dry-run

# Sync a single rule only
rulesify sync --rule typescript-style --dry-run

# Sync from a specific tool only
rulesify sync --tool cursor
```

### Import Examples

Import rules from different AI tools:

```bash
# Import from Cursor (YAML frontmatter + Markdown)
rulesify import --tool cursor .cursor/rules/python-style.mdc

# Import from Cline (Simple Markdown)
rulesify import --tool cline .clinerules/react-patterns.md

# Import from Claude Code (Markdown in project root)
rulesify import --tool claude-code python-style.md

# Import from Goose (Plain text hints)
rulesify import --tool goose coding-standards.goosehints

# Import with custom rule ID
rulesify import --tool cursor .cursor/rules/my-rule.mdc --rule-id custom-name
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
  # auto_apply is now in tool_overrides.cursor section
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
    # Application mode - how Cursor should apply this rule:
    # • always: Apply to every chat and cmd-k session
    # • intelligent: When Agent decides it's relevant (RECOMMENDED)
    # • specific_files: When file matches specified patterns
    # • manual: Only when @-mentioned by user
    apply_mode: intelligent     # Options: always | intelligent | specific_files | manual

    globs: [src/**/*.ts]        # File patterns (only used when apply_mode is "specific_files")
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

## Rule ID Management

Rulesify uses a **unified rule ID system** that ensures consistent, machine-safe identifiers across all operations:

### Rule ID Format
- **Lowercase with hyphens**: `typescript-style`, `react-patterns`
- **Alphanumeric and hyphens only**: Special characters are removed
- **Length limits**: 2-50 characters
- **No consecutive hyphens**: Multiple hyphens are collapsed to single hyphens

### ID Generation Hierarchy
When creating or importing rules, Rulesify determines the rule ID using this priority order:

1. **Embedded HTML comment** (highest priority): `<!-- rulesify-id: custom-id -->`
2. **Filename-based ID**: Extracted from filename and sanitized
3. **Rule name**: Sanitized version of the rule's display name
4. **Timestamp fallback**: `imported-rule-{timestamp}` as last resort

### Sanitization Examples
```bash
"TypeScript Style Guide" → "typescript-style-guide"
"React_Components"       → "react-components"
"My Rule!!!"            → "my-rule"
"rule with   spaces"     → "rule-with-spaces"
```

### ID Tracking in Sync Operations
- **HTML Comment Embedding**: Deployed files include `<!-- rulesify-id: {id} -->` for tracking
- **Filename Preservation**: Sync operations preserve original rule IDs based on `.urf.yaml` filenames
- **Conflict Resolution**: Filename-based IDs take precedence during sync to maintain consistency

This system ensures that rule IDs remain stable across import, export, and sync operations while maintaining compatibility with all supported AI tools.

## Command Reference

### Rule Management
- `rulesify rule new <name>` - Create a new rule from template skeleton
- `rulesify rule edit <name>` - Edit an existing rule in your configured editor
- `rulesify rule list` - List all rules with names and descriptions
- `rulesify rule list -r <regex>` - List rules matching regex pattern (e.g., `-r "test.*"`)
- `rulesify rule show <name>` - Show detailed rule information including content and metadata
- `rulesify rule delete <name>` - Delete a rule (requires confirmation)

### Skills Management
- `rulesify skill list` - List all available skills from registry
- `rulesify skill list --domain <domain>` - Filter skills by domain (development, planning, etc.)
- `rulesify skill list --tags <tags>` - Filter skills by tags (comma-separated)
- `rulesify skill show <skill-id>` - Show detailed skill information
- `rulesify skill add <skill-id>` - Install a skill (downloads SKILL.md or outputs install command)
- `rulesify skill remove <skill-id>` - Remove an installed skill
- `rulesify skill update` - Refresh the local registry cache

### Validation
- `rulesify validate [rule]` - Validate specific rule for quality and format compliance
- `rulesify validate --all` - Validate all rules for issues, warnings, and best practices

### Deployment
- `rulesify deploy --all` - Deploy all rules to all configured default tools
- `rulesify deploy --tool <tool> --all` - Deploy all rules to a specific tool (cursor, cline, claude-code, goose)
- `rulesify deploy --tool <tool> --rule <rule>` - Deploy a specific rule to a specific tool
- `rulesify deploy --tool <tool> --rule <rule1,rule2,rule3>` - Deploy multiple rules (triggers merge) - prompts for merged rule ID

### Synchronization
**Sync detects changes in deployed tool files and updates the corresponding URF files to maintain consistency.**
- `rulesify sync [--dry-run]` - Synchronize all deployed rules back to URF format (use --dry-run to preview changes)
- `rulesify sync --rule <name>` - Sync changes for a specific rule only
- `rulesify sync --tool <tool>` - Sync changes from a specific tool only
- **Note**: Sync reads deployed files (.mdc, .md, .goosehints) and updates URF files when differences are detected

### Configuration
- `rulesify config show` - Show current configuration (storage path, editor, default tools)
- `rulesify config edit` - Edit configuration file in your default editor
- `rulesify config set-storage <path>` - Change the directory where URF files are stored
- `rulesify config set-editor <editor>` - Set default editor (cursor, nano, vim, etc.)
- `rulesify config add-tool <tool>` - Add a tool to default deployment list
- `rulesify config remove-tool <tool>` - Remove a tool from default deployment list

### Import
- `rulesify import --tool <tool> <file>` - Import existing rule from AI tool format to URF
- `rulesify import --tool <tool> <file> --rule-id <custom-id>` - Import with custom rule ID

### Shell Completion
- `rulesify completion <shell>` - Generate shell completion script (bash, zsh, fish, etc.)

**Global Options (available on all commands):**
- `--config <path>` - Use custom configuration file
- `--verbose` - Enable detailed output for debugging

## Example Usage Scenarios

### Author & Deploy a New Rule
```bash
# Create a new rule and open it in your editor
rulesify rule new react-best-practices

# Validate the rule for quality and compliance
rulesify validate react-best-practices

# Deploy to a specific tool (cursor)
rulesify deploy --tool cursor --rule react-best-practices

# Deploy to multiple tools
rulesify deploy --tool cline --rule react-best-practices
rulesify deploy --tool claude-code --rule react-best-practices
```

### Merge & Deploy Multiple Rules
**When deploying multiple rules, rulesify automatically merges them based on priority and prompts for a new rule ID.**

```bash
# Deploy multiple rules - triggers interactive merge
rulesify deploy --tool claude-code --rule "typescript-style,react-patterns,testing-standards"

# Example interaction:
# 📦 Multiple rules detected for merging:
#   1. typescript-style
#   2. react-patterns
#   3. testing-standards
#
# 📋 Merge Preview:
# Rules will be combined in priority order (highest first):
#   1. typescript-style (priority: 8)
#   2. testing-standards (priority: 6)
#   3. react-patterns (priority: 4)
# 📋 Combined tags: typescript, style, testing, react, components
#
# 🔗 Enter ID for the merged rule: full-frontend-guide
# ✅ Merged 3 rules → CLAUDE.md

# The merged rule combines:
# - Metadata from highest priority rule (typescript-style)
# - Descriptions concatenated with separators
# - Tags deduplicated across all rules
# - Content sections in priority order
# - Tool overrides from highest priority rule
```

### Sync Deployed Rules Back to URF
```bash
# Preview what changes sync would make (dry-run)
rulesify sync --dry-run

# Apply synchronization to update URF files from deployed tool files
rulesify sync

# Sync changes from a specific tool only
rulesify sync --tool cursor --dry-run
rulesify sync --tool cursor
```

### Import an Existing Tool Rule
```bash
# Import a Cursor rule
author_rule=.cursor/rules/coding-standards.mdc
rulesify import --tool cursor "$author_rule"

# Deploy the imported rule to other tools
rulesify deploy --tool claude-code --rule coding-standards
rulesify deploy --tool cline --rule coding-standards

# Import with a custom rule ID
rulesify import --tool cline .clinerules/my-rule.md --rule-id custom-rule-name
```

### Share Rules Across the Team
```bash
# Validate all rules before sharing
rulesify validate --all

# Deploy all rules to your preferred tools
rulesify deploy --all

# Commit URF files to version control and push to the repository
git add rules/*.urf.yaml
git commit -m "Add/Update team coding rules"
git push origin main

# Team members can then pull, validate, and deploy
git pull origin main
rulesify validate --all
rulesify deploy --all
```

### Advanced Workflow Examples
```bash
# List rules matching a pattern
rulesify rule list -r "typescript.*"

# Show detailed information about a specific rule
rulesify rule show typescript-style

# Deploy all rules to just one tool
rulesify deploy --tool cursor --all

# Sync and validate workflow
rulesify sync --dry-run    # Preview changes
rulesify sync              # Apply changes
rulesify validate --all    # Ensure quality
rulesify deploy --all      # Deploy updates

# Merge and deploy multiple rules for comprehensive tooling
rulesify deploy --tool claude-code --rule "typescript-style,react-patterns,testing-standards"
```

### Skills Workflow Examples
```bash
# Discover available skills
rulesify skill list

# Find skills for debugging
rulesify skill list --tags debugging

# Install a development skill
rulesify skill add tdd

# Install a complex skill (outputs command to run)
rulesify skill add gsd
# Output: Run: git clone https://github.com/gsd-build/get-shit-done ~/.agents/skills/gsd

# View skill details before installing
rulesify skill show systematic-debugging
```

### Interactive Features

**Confirmation Prompts:**
- `rulesify rule delete <name>` - Requires confirmation before deletion
- Import commands ask if you want to open the rule in editor after import
- Multi-rule deployment prompts for merged rule ID and overwrite confirmation

**Editor Integration:**
- `rulesify rule new <name>` - Automatically opens new rule in configured editor
- `rulesify rule edit <name>` - Opens existing rule for editing
- `rulesify config edit` - Opens configuration file for editing

**Verbose Output:**
- Add `--verbose` to any command for detailed debugging information
- Use `--config <path>` to specify a custom configuration file location

## Development

See [`DEVELOPMENT_PLAN_DETAILED.md`](./DEVELOPMENT_PLAN_DETAILED.md) for the complete development roadmap and architecture details.

### Project Structure
```
rulesify/
├── Cargo.toml
├── registry.toml              # Skills registry catalog (with LLM-derived domains/tags)
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── bin/
│   │   └── update-registry.rs # Registry automation binary (LLM classification)
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands/
│   ├── models/
│   │   ├── domain.rs          # Domain enum (10 predefined domains)
│   │   ├── skill.rs           # Skill model
│   │   ├── registry.rs        # Registry model
│   │   ├── install_action.rs  # Install types (copy/command)
│   │   └── skill_metadata.rs  # Raw skill data
│   ├── llm/
│   │   ├── mod.rs             # LLM module exports
│   │   ├── client.rs          # OpenRouter HTTP client
│   │   ├── classifier.rs      # Batch classification (20 skills/request)
│   │   └── prompt.rs          # System/user prompt builder
│   ├── registry/
│   │   ├── source.rs          # GitHub repo sources
│   │   ├── github.rs          # GitHub API client
│   │   ├── parser.rs          # SKILL.md parser
│   │   ├── scorer.rs          # Quality scoring
│   │   └── generator.rs       # TOML generator
│   ├── store/
│   ├── converters/
│   ├── templates/
│   ├── validation/
│   ├── sync/
│   └── utils/
├── templates/
├── tests/
├── .github/
│   └── workflows/
│       └── update-registry.yml # Weekly registry updates
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
- **38 total tests** across all modules
- **10 domain tests** (Domain enum parsing, validation, serialization)
- **24 registry tests** (skills catalog, GitHub API, scoring)
- **19 converter tests** (import/export functionality)
- **22 validation tests** (quality assurance)
- **10+ CLI integration tests** (including multi-rule merge)
- **11 import tests**
- **4 end-to-end tests** (complete workflows)
- **5 storage tests** (persistence layer)
- **6 template tests** (rule generation)
- **Unicode and round-trip integrity tested**

## License

This project is licensed under the [MIT License](./LICENSE).

Copyright (c) 2024 Rulesify Contributors
