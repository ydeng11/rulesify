# Rulesify

Rulesify is a command-line tool (written in Rust) that provides **unified management of AI assistant rules** across multiple platforms. Create rules once in Universal Rule Format (URF), then deploy them everywhere.

## Supported AI Tools

- **Cursor** (`.cursor/rules/*.mdc`)
- **Cline** (`.clinerules/*.md`)
- **Claude Code** (`CLAUDE.md`)
- **Goose** (`.goosehints`)

## Features

✅ **Rule Management**: Create, edit, list, show, and delete rules
✅ **Multi-Tool Deployment**: Deploy rules to all supported AI tools
✅ **Import Functionality**: Import existing rules from any AI tool format
✅ **Validation System**: Comprehensive quality and format validation
✅ **Template System**: Create rules from built-in templates
✅ **Configuration Management**: Flexible storage and configuration options
🚧 **Sync**: Synchronize deployed rules back to URF format

## Quick Start

### Installation
```bash
cargo build --release
```

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

Rulesify uses a JSON-based Universal Rule Format for internal storage:

```json
{
  "id": "typescript-style",
  "version": "1.0.0",
  "metadata": {
    "name": "TypeScript Style Guide",
    "description": "Coding standards for TypeScript projects",
    "tags": ["typescript", "style", "linting"],
    "priority": 5,
    "auto_apply": true
  },
  "content": [
    {
      "title": "Code Style",
      "format": "Markdown",
      "value": "Use consistent formatting..."
    }
  ],
  "references": [
    {"path": "tsconfig.json"}
  ],
  "conditions": [
    {"FilePattern": {"value": "**/*.ts"}}
  ]
}
```

## Command Reference

### Rule Management
- `rule new <name>` - Create a new rule
- `rule edit <name>` - Edit an existing rule
- `rule list` - List all rules
- `rule show <name>` - Show rule details
- `rule delete <name>` - Delete a rule

### Import & Validation
- `import --tool <tool> <file>` - Import rule from AI tool format
- `validate [rule]` - Validate rule(s)
- `validate --all` - Validate all rules

### Deployment
- `deploy --all` - Deploy all rules to all tools
- `deploy --tool <tool> --all` - Deploy to specific tool
- `deploy --tool <tool> <rule>` - Deploy specific rule to tool

### Configuration
- `config show` - Show current configuration
- `config edit` - Edit configuration file

## Development

See [`DEVELOPMENT_PLAN_DETAILED.md`](./DEVELOPMENT_PLAN_DETAILED.md) for the complete development roadmap and architecture details.

### Testing
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test converter_tests
cargo test --test validation_tests
```

### Test Coverage
- **56 total tests** across all modules
- **19 converter tests** (import/export functionality)
- **22 validation tests** (quality assurance)
- **4 end-to-end tests** (complete workflows)
- **5 storage tests** (persistence layer)
- **6 template tests** (rule generation)
